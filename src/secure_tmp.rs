use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns a verified owner-only per-user runtime directory.
///
/// Priority:
/// 1. `$XDG_RUNTIME_DIR/omarchy-pie` (typically `/run/user/<uid>/omarchy-pie`, mode 0700 tmpfs)
/// 2. `/tmp/omarchy-pie-<uid>` (mode 0700, strictly owned by current user)
///
/// Security verifications:
/// - Refuses to follow any symlinks.
/// - Verifies owner matches current process UID.
/// - Ensures directory mode is strictly 0700 (owner only: rwx------).
pub fn get_secure_runtime_dir() -> Result<PathBuf, String> {
    let current_uid = unsafe { libc::getuid() };

    let candidate = if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !xdg_runtime.trim().is_empty() {
            PathBuf::from(xdg_runtime).join("omarchy-pie")
        } else {
            std::env::temp_dir().join(format!("omarchy-pie-{}", current_uid))
        }
    } else {
        std::env::temp_dir().join(format!("omarchy-pie-{}", current_uid))
    };

    ensure_secure_dir(&candidate, current_uid)?;
    Ok(candidate)
}

fn verify_dir_metadata(path: &Path, expected_uid: u32) -> Result<(), String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|e| format!("Failed to read metadata for {:?}: {}", path, e))?;

    // Must never be a symlink (avoids symlink hijacking attacks)
    if meta.file_type().is_symlink() {
        return Err(format!(
            "Security violation: {:?} is a symlink, refusing to use",
            path
        ));
    }

    // Must be a directory
    if !meta.is_dir() {
        return Err(format!(
            "Security violation: {:?} exists but is not a directory",
            path
        ));
    }

    // Must be owned by current user
    if meta.uid() != expected_uid {
        return Err(format!(
            "Security violation: {:?} is owned by UID {} (expected {})",
            path,
            meta.uid(),
            expected_uid
        ));
    }

    // Must be restricted to owner only (0700)
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        let _ = fs::set_permissions(path, Permissions::from_mode(0o700));
        let updated_meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        let updated_mode = updated_meta.permissions().mode() & 0o777;
        if updated_mode & 0o077 != 0 {
            return Err(format!(
                "Security violation: {:?} permissions {:o} are not owner-only",
                path, updated_mode
            ));
        }
    }

    Ok(())
}

fn ensure_secure_dir(path: &Path, expected_uid: u32) -> Result<(), String> {
    if path.exists() {
        return verify_dir_metadata(path, expected_uid);
    }

    // Directory does not exist: create with mode 0700 (owner only)
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder.recursive(false); // Do not recursively build parent path with unknown permissions
    match builder.create(path) {
        Ok(_) => verify_dir_metadata(path, expected_uid),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            verify_dir_metadata(path, expected_uid)
        }
        Err(e) => Err(format!("Failed to create secure runtime directory {:?}: {}", path, e)),
    }
}

/// Creates a new private file exclusively (`O_CREAT | O_EXCL`) with owner-only
/// permissions (`mode 0600`).
///
/// Refuses to follow pre-created files or symlinks.
pub fn create_private_file(
    dir: &Path,
    prefix: &str,
    ext: &str,
) -> Result<(File, PathBuf), String> {
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    for attempt in 0..1000 {
        let filename = format!("{}_{}_{}_{}.{}", prefix, pid, now, attempt, ext);
        let path = dir.join(filename);

        match OpenOptions::new()
            .write(true)
            .create_new(true) // O_CREAT | O_EXCL: Fails atomically if file or symlink exists!
            .mode(0o600)      // Owner-only: rw-------
            .open(&path)
        {
            Ok(file) => return Ok((file, path)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Failed to create private file {:?}: {}", path, e)),
        }
    }

    Err("Exhausted unique attempts to create private file".to_string())
}

/// Atomically writes data to an exclusively created private file with mode 0600.
pub fn write_private_file(
    dir: &Path,
    prefix: &str,
    ext: &str,
    data: &[u8],
) -> Result<PathBuf, String> {
    let (mut file, path) = create_private_file(dir, prefix, ext)?;
    file.write_all(data)
        .map_err(|e| format!("Failed to write private file {:?}: {}", path, e))?;
    file.flush()
        .map_err(|e| format!("Failed to flush private file {:?}: {}", path, e))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_runtime_dir_permissions() {
        let dir = get_secure_runtime_dir().expect("secure runtime dir creation should succeed");
        assert!(dir.exists(), "directory must exist");

        let meta = fs::symlink_metadata(&dir).expect("must get symlink metadata");
        assert!(!meta.file_type().is_symlink(), "must not be a symlink");
        assert!(meta.is_dir(), "must be a directory");

        let uid = unsafe { libc::getuid() };
        assert_eq!(meta.uid(), uid, "must be owned by current user");

        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode & 0o077, 0, "group and others must have 0 permissions");
    }

    #[test]
    fn test_private_file_creation_mode_0600() {
        let dir = get_secure_runtime_dir().expect("secure runtime dir creation should succeed");
        let path = write_private_file(&dir, "test_file", "tmp", b"secure payload")
            .expect("writing private file should succeed");

        assert!(path.exists());
        let meta = fs::symlink_metadata(&path).expect("must get symlink metadata");
        assert!(!meta.file_type().is_symlink());

        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "file mode must be strictly 0600 (rw-------)");

        let content = fs::read(&path).expect("read content");
        assert_eq!(content, b"secure payload");

        let _ = fs::remove_file(path);
    }
}

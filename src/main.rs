mod actions;
mod context;

use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct CursorPos {
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct MonitorInfo {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f64,
    focused: bool,
}

#[derive(Deserialize)]
struct ActiveWindow {
    #[serde(default)]
    class: String,
    #[serde(default)]
    pid: u32,
}

fn get_cursor_and_monitor() -> (i32, i32, i32, i32) {
    let cursor: CursorPos = Command::new("hyprctl")
        .args(["cursorpos", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice(&o.stdout).ok())
        .unwrap_or(CursorPos { x: 960, y: 540 });

    let monitors: Vec<MonitorInfo> = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice(&o.stdout).ok())
        .unwrap_or_default();

    if let Some(m) = monitors
        .iter()
        .find(|m| m.focused)
        .or_else(|| monitors.first())
    {
        let scale = if m.scale > 0.1 { m.scale } else { 1.0 };
        let logical_w = (m.width as f64 / scale).round() as i32;
        let logical_h = (m.height as f64 / scale).round() as i32;
        let local_x = (cursor.x - m.x).clamp(0, logical_w);
        let local_y = (cursor.y - m.y).clamp(0, logical_h);
        return (local_x, local_y, logical_w, logical_h);
    }

    (cursor.x, cursor.y, 1920, 1080)
}

fn get_active_window() -> Option<ActiveWindow> {
    let out = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

fn parse_candidate_path(raw: &str, cwd: Option<&Path>) -> Option<PathBuf> {
    for line in raw.lines() {
        let trimmed = line.trim().trim_matches('\'').trim_matches('"');
        if trimmed.is_empty() || trimmed == "copy" || trimmed == "cut" || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("file://") {
            let decoded = percent_decode_str(rest).decode_utf8_lossy().to_string();
            let p = PathBuf::from(decoded);
            if p.exists() {
                return Some(p);
            }
        }
        if let Some(rest) = trimmed.strip_prefix("~/") {
            if let Ok(home) = env::var("HOME") {
                let p = PathBuf::from(home).join(rest);
                if p.exists() {
                    return Some(p);
                }
            }
        }
        let p = PathBuf::from(trimmed);
        if p.is_absolute() && p.exists() {
            return Some(p);
        }
        if let Some(base) = cwd {
            let joined = base.join(trimmed);
            if joined.exists() {
                return Some(joined);
            }
        }
    }
    None
}

fn read_clipboard_uri_list(win_cwd: Option<&Path>) -> Option<PathBuf> {
    let types_out = Command::new("wl-paste")
        .arg("--list-types")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    if types_out.contains("text/uri-list") {
        if let Ok(out) = Command::new("wl-paste")
            .args(["--type", "text/uri-list"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(p) = parse_candidate_path(&text, win_cwd) {
                return fs::canonicalize(&p).ok().or(Some(p));
            }
        }
    }

    if types_out.contains("x-special/gnome-copied-files") {
        if let Ok(out) = Command::new("wl-paste")
            .args(["--type", "x-special/gnome-copied-files"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(p) = parse_candidate_path(&text, win_cwd) {
                return fs::canonicalize(&p).ok().or(Some(p));
            }
        }
    }

    None
}

fn resolve_target_file(explicit_arg: Option<&str>) -> Option<PathBuf> {
    // 1. Explicit CLI argument
    if let Some(arg) = explicit_arg {
        if let Some(p) = parse_candidate_path(arg, env::current_dir().ok().as_deref()) {
            return fs::canonicalize(&p).ok().or(Some(p));
        }
    }

    let active_win = get_active_window();
    let win_cwd = active_win
        .as_ref()
        .filter(|w| w.pid > 0)
        .and_then(|w| fs::read_link(format!("/proc/{}/cwd", w.pid)).ok());

    let is_file_manager = active_win
        .as_ref()
        .map(|w| {
            let c = w.class.to_lowercase();
            c.contains("nautilus")
                || c.contains("thunar")
                || c.contains("dolphin")
                || c.contains("nemo")
                || c.contains("pcmanfm")
        })
        .unwrap_or(false);

    // 2. If in file manager (or URI list is on clipboard), prioritize text/uri-list!
    if let Some(p) = read_clipboard_uri_list(win_cwd.as_deref()) {
        return Some(p);
    }

    // 3. If not in a GUI file manager, check primary selection (e.g. highlighted filename in terminal)
    if !is_file_manager {
        if let Ok(out) = Command::new("wl-paste")
            .args(["--primary", "--no-newline"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(p) = parse_candidate_path(&text, win_cwd.as_deref()) {
                    return fs::canonicalize(&p).ok().or(Some(p));
                }
            }
        }
    }

    // 4. Plain text path in clipboard
    if let Ok(out) = Command::new("wl-paste").arg("--no-newline").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(p) = parse_candidate_path(&text, win_cwd.as_deref()) {
                return fs::canonicalize(&p).ok().or(Some(p));
            }
        }
    }

    // 5. Raw screenshot / image in clipboard
    let types_out = Command::new("wl-paste")
        .arg("--list-types")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    if types_out.contains("image/png") {
        let tmp_dir = PathBuf::from("/tmp/omarchy-pie");
        let _ = fs::create_dir_all(&tmp_dir);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let img_path = tmp_dir.join(format!("clipboard_{}.png", ts));
        if let Ok(out) = Command::new("wl-paste")
            .args(["--type", "image/png"])
            .output()
        {
            if out.status.success() && !out.stdout.is_empty() {
                if fs::write(&img_path, &out.stdout).is_ok() {
                    return Some(img_path);
                }
            }
        }
    }

    // 6. Fallback: Most recently modified file in ~/Downloads or ~/Pictures
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home);
        let search_dirs = [
            home_path.join("Downloads"),
            home_path.join("Pictures"),
            home_path.clone(),
        ];
        let mut newest: Option<(SystemTime, PathBuf)> = None;
        for dir in &search_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.starts_with('.') || !p.is_file() {
                        continue;
                    }
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if newest.as_ref().map_or(true, |(t, _)| modified > *t) {
                                newest = Some((modified, p));
                            }
                        }
                    }
                }
            }
            if newest.is_some() {
                break;
            }
        }
        if let Some((_, p)) = newest {
            return Some(p);
        }
    }

    None
}

fn summon_pie(explicit_path: Option<&str>, print_only: bool) {
    let Some(target) = resolve_target_file(explicit_path) else {
        actions::notify(
            "Omarchy Pie",
            "Selecciona o copia un archivo primero para abrir el menú radial.",
        );
        return;
    };

    let (cx, cy, mw, mh) = get_cursor_and_monitor();
    let payload = context::build_payload(&target, cx, cy, mw, mh);
    let json_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());

    if print_only {
        println!("{}", json_str);
        return;
    }

    let mut summoned = false;
    for id in ["io.github.jeffcortez23.omarchy-pie", "elyefris.pie"] {
        if let Ok(out) = Command::new("/usr/share/omarchy/bin/omarchy-shell")
            .env("OMARCHY_PATH", "/usr/share/omarchy")
            .args(["shell", "summon", id, &json_str])
            .output()
        {
            let reply = String::from_utf8_lossy(&out.stdout);
            if out.status.success() && reply.trim() == "ok" {
                summoned = true;
                break;
            }
        }
    }

    if !summoned {
        actions::notify(
            "Omarchy Pie Error",
            "No se pudo invocar el plugin io.github.jeffcortez23.omarchy-pie en omarchy-shell",
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        summon_pie(None, false);
        return;
    }

    match args[1].as_str() {
        "summon" | "open" => {
            let file_arg = args.get(2).map(|s| s.as_str());
            summon_pie(file_arg, false);
        }
        "inspect" => {
            let file_arg = args.get(2).map(|s| s.as_str());
            summon_pie(file_arg, true);
        }
        "exec" => {
            if args.len() < 4 {
                eprintln!("Uso: omarchy-pie exec <action_id> <file_path>");
                std::process::exit(1);
            }
            let action_id = &args[2];
            let file_path = Path::new(&args[3]);
            if let Err(e) = actions::execute_action(action_id, file_path) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "--help" | "-h" | "help" => {
            println!("omarchy-pie — Menú radial contextual de archivos para Omarchy");
            println!();
            println!("Uso:");
            println!("  omarchy-pie [archivo]               Detecta el archivo seleccionado y abre el menú radial en el cursor");
            println!("  omarchy-pie summon [archivo]        Abre el menú radial en Quickshell");
            println!("  omarchy-pie inspect [archivo]       Imprime el JSON de contexto y acciones detectadas");
            println!("  omarchy-pie exec <accion> <archivo> Ejecuta una acción directamente sobre el archivo");
        }
        other_path => {
            summon_pie(Some(other_path), false);
        }
    }
}

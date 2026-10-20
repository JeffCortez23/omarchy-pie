use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieSlice {
    pub id: String,
    pub label: String,
    pub subtitle: String,
    pub icon: String,
    pub category: String,
    pub keeps_open: bool,
    #[serde(default)]
    pub sub_slices: Vec<PieSlice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiePayload {
    pub file_path: String,
    pub file_name: String,
    pub file_size: String,
    pub file_kind: String,
    pub file_icon: String,
    pub is_image: bool,
    pub dimensions: String,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub monitor_width: i32,
    pub monitor_height: i32,
    pub slices: Vec<PieSlice>,
}

pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn slice(
    id: &str,
    label: &str,
    subtitle: &str,
    icon: &str,
    category: &str,
    keeps_open: bool,
) -> PieSlice {
    PieSlice {
        id: id.to_string(),
        label: label.to_string(),
        subtitle: subtitle.to_string(),
        icon: icon.to_string(),
        category: category.to_string(),
        keeps_open,
        sub_slices: Vec::new(),
    }
}

fn menu_slice(
    id: &str,
    label: &str,
    subtitle: &str,
    icon: &str,
    category: &str,
    sub_slices: Vec<PieSlice>,
) -> PieSlice {
    PieSlice {
        id: id.to_string(),
        label: label.to_string(),
        subtitle: subtitle.to_string(),
        icon: icon.to_string(),
        category: category.to_string(),
        keeps_open: false,
        sub_slices,
    }
}

fn share_submenu() -> PieSlice {
    menu_slice(
        "menu_share",
        "Share ▸",
        "Send via LocalSend, Wi-Fi QR, or cloud link",
        "󰐲",
        "share",
        vec![
            slice(
                "send_localsend",
                "LocalSend",
                "Send to nearby devices via LocalSend",
                "󰄙",
                "share",
                false,
            ),
            slice(
                "share_qr",
                "Wi-Fi QR",
                "Local HTTP server with instant QR code",
                "󰐲",
                "share",
                true,
            ),
            slice(
                "upload_0x0",
                "Upload 0x0.st",
                "Upload to cloud and generate link + QR",
                "󰇚",
                "share",
                true,
            ),
            slice(
                "copy_path",
                "Copy Path",
                "Copy absolute path to clipboard",
                "󰅍",
                "dev",
                false,
            ),
            slice(
                "copy_data_uri",
                "Copy Base64",
                "Encode file as Data URI / Base64",
                "󰘦",
                "dev",
                false,
            ),
        ],
    )
}

fn normalize_name_submenu() -> PieSlice {
    menu_slice(
        "menu_normalize",
        "Normalize ▸",
        "Rename with clean format (kebab, snake, date…)",
        "󰑕",
        "dev",
        vec![
            slice(
                "norm_kebab",
                "kebab-case",
                "my-clean-file.ext (clean URL-friendly)",
                "󰑕",
                "dev",
                false,
            ),
            slice(
                "norm_snake",
                "snake_case",
                "my_clean_file.ext (underscores)",
                "󰌌",
                "dev",
                false,
            ),
            slice(
                "norm_camel",
                "camelCase",
                "myCleanFile.ext (programming style)",
                "󰅩",
                "dev",
                false,
            ),
            slice(
                "norm_date",
                "Date Prefix",
                "Add YYYY-MM-DD_ prefix to filename",
                "󰃭",
                "dev",
                false,
            ),
            slice(
                "norm_clean",
                "Clean Clutter",
                "Remove (1), Copy, WhatsApp, brackets",
                "󰃢",
                "tools",
                false,
            ),
            slice(
                "norm_lower",
                "lowercase",
                "Convert entire filename to lowercase",
                "󰬵",
                "dev",
                false,
            ),
        ],
    )
}

fn image_scale_submenu() -> PieSlice {
    menu_slice(
        "menu_scale",
        "Scale ▸",
        "Resize dimensions or resolution of image",
        "󰩨",
        "convert",
        vec![
            slice(
                "scale_50",
                "50% Scale",
                "Reduce width and height by half",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_75",
                "75% Scale",
                "Smooth downscale preserving sharpness",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_25",
                "25% Scale",
                "Lightweight thumbnail (1/4 size)",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_1080",
                "Max 1080p",
                "Fit longest side to 1920px (Full HD)",
                "󰍹",
                "convert",
                false,
            ),
            slice(
                "scale_720",
                "Max 720p",
                "Fit longest side to 1280px (HD)",
                "󰍹",
                "convert",
                false,
            ),
            slice(
                "scale_square",
                "Avatar 512×512",
                "Square centered 1:1 crop",
                "󰆞",
                "convert",
                false,
            ),
        ],
    )
}

fn image_convert_submenu(ext: &str) -> PieSlice {
    let mut subs = Vec::new();
    if ext != "webp" {
        subs.push(slice("to_webp", "To WebP", "Ultra-lightweight modern format", "󰋩", "convert", false));
    }
    if ext != "png" {
        subs.push(slice("to_png", "To PNG", "Lossless format with transparency", "󰸭", "convert", false));
    }
    if ext != "jpg" && ext != "jpeg" {
        subs.push(slice("to_jpg", "To JPG", "Universal compatible JPEG", "󰈥", "convert", false));
    }
    if ext != "avif" {
        subs.push(slice("to_avif", "To AVIF", "Next-gen ultra-high compression", "󰋩", "convert", false));
    }
    subs.push(slice("to_ico", "To Favicon .ICO", "64×64 icon for web & apps", "󰀻", "convert", false));
    subs.push(slice("img_grayscale", "Grayscale", "Convert to black and white", "󰏘", "tools", false));

    menu_slice(
        "menu_convert_img",
        "Convert ▸",
        "Convert to WebP, PNG, JPG, AVIF, ICO or Grayscale",
        "󰋩",
        "convert",
        subs,
    )
}

fn image_transform_submenu() -> PieSlice {
    menu_slice(
        "menu_transform_img",
        "Transform ▸",
        "Rotate, flip, strip EXIF or extract colors",
        "󰑵",
        "tools",
        vec![
            slice("rot_90_cw", "Rotate 90° ↻", "Rotate 90 degrees clockwise", "󰑵", "tools", false),
            slice("rot_90_ccw", "Rotate 90° ↺", "Rotate 90 degrees counter-clockwise", "󰑴", "tools", false),
            slice("rot_180", "Rotate 180°", "Flip orientation vertically", "󰑵", "tools", false),
            slice("flip_h", "Mirror Flip", "Flip image horizontally", "󰹳", "tools", false),
            slice("extract_palette", "HEX Palette", "Extract 5 dominant colors to clipboard", "󰏘", "dev", false),
            slice("strip_exif", "Strip EXIF", "Remove GPS and camera metadata", "󰒃", "privacy", false),
        ],
    )
}

pub fn build_payload(
    path: &Path,
    cursor_x: i32,
    cursor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
) -> PiePayload {
    let file_path = path.to_string_lossy().to_string();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let metadata = fs::metadata(path).ok();
    let file_size = metadata
        .as_ref()
        .map(|m| {
            if m.is_dir() {
                let count = fs::read_dir(path).map(|d| d.count()).unwrap_or(0);
                format!("{} items", count)
            } else {
                format_size(m.len())
            }
        })
        .unwrap_or_else(|| "0 B".to_string());

    if path.is_dir() {
        return PiePayload {
            file_path,
            file_name,
            file_size,
            file_kind: "FOLDER".to_string(),
            file_icon: "󰉋".to_string(),
            is_image: false,
            dimensions: String::new(),
            cursor_x,
            cursor_y,
            monitor_width,
            monitor_height,
            slices: vec![
                slice("archive_zip", "Create .ZIP", "Compress folder into .zip", "󰗄", "convert", false),
                slice("archive_zst", "Create .zst", "Compress with ultra-fast Zstandard", "󰀼", "convert", false),
                normalize_name_submenu(),
                slice("send_localsend", "LocalSend", "Send folder to nearby device", "󰄙", "share", false),
                share_submenu(),
                slice("open_terminal", "Terminal Here", "Open terminal in this directory", "󰆍", "dev", false),
                slice("open_nvim", "In Neovim", "Open folder as project in Neovim", "", "dev", false),
                slice("sha256_sum", "Total Size", "Calculate total folder disk usage", "󰋊", "tools", false),
            ],
        };
    }

    let name_lower = file_name.to_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut dimensions = String::new();
    if matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "ico" | "tiff"
    ) {
        if let Ok((w, h)) = image::image_dimensions(path) {
            dimensions = format!("{}×{}", w, h);
        }
    }

    let (file_kind, file_icon, is_image, slices) = match ext.as_str() {
        // 1. Raster Images (PNG, JPG, WebP, AVIF, BMP, GIF, TIFF, ICO)
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "bmp" | "gif" | "tiff" | "ico" => (
            format!("IMAGE {}", ext.to_uppercase()),
            "󰋩".to_string(),
            ext != "avif",
            vec![
                image_convert_submenu(&ext),
                image_scale_submenu(),
                slice(
                    "compress_img",
                    "Compress Weight",
                    "Reduce MB/KB keeping exact same resolution",
                    "󰗄",
                    "convert",
                    false,
                ),
                image_transform_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send image via LocalSend", "󰄙", "share", false),
                slice("ocr_text", "OCR Text", "Extract text with Tesseract", "󰗊", "tools", false),
                normalize_name_submenu(),
            ],
        ),

        // 2. Vector SVG
        "svg" => (
            "VECTOR SVG".to_string(),
            "󰜡".to_string(),
            true,
            vec![
                slice("to_png", "Render PNG", "Export SVG vector to PNG", "󰸭", "convert", false),
                slice("to_webp", "Render WebP", "Export SVG vector to WebP", "󰋩", "convert", false),
                slice("copy_data_uri", "Data URI SVG", "Copy data:image/svg+xml;base64", "󰅍", "dev", false),
                slice("copy_content", "Copy XML", "Copy SVG source code to clipboard", "󰘦", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send SVG via LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("open_nvim", "Edit SVG", "Open SVG source in Neovim", "", "dev", false),
            ],
        ),

        // 3. Video files
        "mp4" | "mkv" | "mov" | "webm" | "avi" | "flv" | "m4v" => (
            format!("VIDEO {}", ext.to_uppercase()),
            "󰕧".to_string(),
            false,
            vec![
                slice("video_to_gif", "To GIF", "Convert to animated GIF with palette", "󰵸", "convert", false),
                slice("extract_mp3", "Extract MP3", "Extract audio track to MP3", "󰎆", "convert", false),
                slice("compress_discord", "Compress", "Reduce file size in H.264 (<720p)", "󰙯", "convert", false),
                slice("mute_video", "Mute Audio", "Remove audio track without re-encoding", "󰝟", "tools", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send video via LocalSend", "󰄙", "share", false),
                slice("video_thumbnail", "Capture Frame", "Extract snapshot keyframe as PNG", "󰄀", "tools", false),
                normalize_name_submenu(),
            ],
        ),

        // 4. Audio files
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" => (
            format!("AUDIO {}", ext.to_uppercase()),
            "󰎆".to_string(),
            false,
            vec![
                if ext == "mp3" {
                    slice("audio_to_opus", "To Opus", "Compress to lightweight Opus", "󰋋", "convert", false)
                } else {
                    slice("extract_mp3", "To MP3", "Convert to MP3 192kbps", "󰎆", "convert", false)
                },
                if ext == "wav" {
                    slice("audio_to_flac", "To FLAC", "Compress losslessly to FLAC", "󰕾", "convert", false)
                } else {
                    slice("audio_to_wav", "To WAV", "Convert to uncompressed PCM WAV", "󰕾", "convert", false)
                },
                slice("audio_normalize", "Normalize Vol", "Normalize loudness leveling (loudnorm)", "󰓃", "tools", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send audio via LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Copy SHA-256 checksum to clipboard", "󰒃", "dev", false),
                slice("copy_path", "Copy Path", "Copy absolute path to clipboard", "󰅍", "dev", false),
            ],
        ),

        // 5. PDF Documents
        "pdf" => (
            "PDF DOCUMENT".to_string(),
            "󰈦".to_string(),
            false,
            vec![
                slice("pdf_to_png", "To Images", "Export PDF pages to PNG images", "󰸭", "convert", false),
                slice("archive_zip", "To .ZIP", "Package PDF into ZIP archive", "󰗄", "convert", false),
                slice("archive_zst", "To .tar.zst", "Compress with Zstandard", "󰀼", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send PDF via LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Calculate SHA-256 checksum", "󰒃", "dev", false),
                slice("copy_path", "Copy Path", "Copy PDF file path", "󰅍", "dev", false),
            ],
        ),

        // 6. CSV / TSV Data tables
        "csv" | "tsv" => (
            format!("DATA {}", ext.to_uppercase()),
            "󰈛".to_string(),
            false,
            vec![
                slice("csv_to_json", "To JSON", "Convert CSV table to JSON file", "󰘦", "convert", false),
                slice("file_stats", "Count Rows", "View total records and columns", "󰄨", "tools", false),
                slice("archive_zst", "Compress .zst", "Compress dataset with Zstandard", "󰀼", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send dataset via LocalSend", "󰄙", "share", false),
                slice("open_nvim", "In Neovim", "Inspect dataset in Neovim", "", "dev", false),
                slice("sha256_sum", "SHA-256", "Calculate SHA-256 integrity hash", "󰒃", "dev", false),
            ],
        ),

        // 7. JSON / TOML / YAML Configs
        "json" | "jsonc" | "toml" | "yaml" | "yml" => (
            format!("CONFIG {}", ext.to_uppercase()),
            "󰘦".to_string(),
            false,
            vec![
                if ext.starts_with("json") {
                    slice("json_pretty", "Format JSON", "Beautify and indent JSON", "󰉼", "tools", false)
                } else {
                    slice("file_stats", "Count Lines", "Line and byte statistics", "󰈙", "tools", false)
                },
                if ext.starts_with("json") {
                    slice("json_minify", "Minify JSON", "Compact JSON into one line", "󰗄", "convert", false)
                } else {
                    slice("archive_zip", "To .ZIP", "Package into ZIP archive", "󰗄", "convert", false)
                },
                slice("copy_content", "Copy All", "Copy content to clipboard", "󰅍", "dev", false),
                slice("open_nvim", "In Neovim", "Edit configuration in Neovim", "", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send file via LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("copy_data_uri", "Base64", "Copy Base64 encoded string", "󰨿", "dev", false),
            ],
        ),

        // 8. Markdown & Text
        "md" | "txt" | "rst" | "log" => (
            format!("TEXT {}", ext.to_uppercase()),
            "󰍔".to_string(),
            false,
            vec![
                if ext == "md" {
                    slice("md_to_html", "To HTML", "Export Markdown to styled HTML page", "󰌝", "convert", false)
                } else {
                    slice("archive_zst", "To .tar.zst", "Compress text with Zstandard", "󰀼", "convert", false)
                },
                slice("copy_content", "Copy Text", "Copy entire content to clipboard", "󰅍", "dev", false),
                slice("file_stats", "Count Words", "View lines, words, and characters", "󰈙", "tools", false),
                slice("open_nvim", "In Neovim", "Edit document in Neovim", "", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send document via LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("archive_zip", "To .ZIP", "Compress document into ZIP", "󰗄", "convert", false),
            ],
        ),

        // 9. Scripts & Source Code
        "sh" | "bash" | "zsh" | "py" | "rs" | "go" | "js" | "ts" | "lua" | "qml" | "c" | "cpp" | "html" | "css" => (
            format!("CODE {}", ext.to_uppercase()),
            "󰅩".to_string(),
            false,
            vec![
                slice("open_nvim", "In Neovim", "Open code in Neovim", "", "dev", false),
                slice("chmod_x", "Make +x", "Grant execute permissions (chmod +x)", "󰒓", "tools", false),
                slice("run_script", "Run Script", "Run script in new terminal window", "󰆍", "dev", false),
                slice("copy_content", "Copy Code", "Copy source code to clipboard", "󰅍", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send code via LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("file_stats", "Lines (LOC)", "Count lines of code", "󰈙", "tools", false),
            ],
        ),

        // 10. Blender / 3D Files
        "blend" | "blend1" | "obj" | "fbx" | "stl" | "gltf" | "glb" => (
            format!("3D {}", ext.to_uppercase()),
            "󰆧".to_string(),
            false,
            vec![
                slice("open_blender", "Open Blender", "Open scene in Blender", "󰂫", "tools", false),
                slice("archive_zst", "To .tar.zst", "Compress 3D project with Zstandard", "󰀼", "convert", false),
                slice("archive_zip", "To .ZIP", "Package 3D project in ZIP", "󰗄", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send 3D model via LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Calculate model SHA-256 hash", "󰒃", "dev", false),
                slice("open_terminal", "Terminal Here", "Open terminal in this directory", "󰆍", "dev", false),
            ],
        ),

        // 11. Archives & Compressed Packages
        "zip" | "tar" | "gz" | "zst" | "xz" | "bz2" | "7z" | "rar" | "tgz" => (
            format!("ARCHIVE {}", ext.to_uppercase()),
            "󰀼".to_string(),
            false,
            vec![
                slice("extract_archive", "Extract Here", "Decompress into clean subfolder", "󰿺", "tools", false),
                slice("list_archive", "View Files", "Count entries inside archive", "󰈙", "tools", false),
                slice("sha256_sum", "SHA-256", "Verify SHA-256 checksum", "󰒃", "dev", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send archive via LocalSend", "󰄙", "share", false),
                slice("copy_path", "Copy Path", "Copy absolute path to clipboard", "󰅍", "dev", false),
                slice("open_terminal", "Terminal Here", "Open terminal in this directory", "󰆍", "dev", false),
            ],
        ),

        // 12. AppImage / Binaries
        "appimage" | "run" | "bin" => (
            "EXECUTABLE".to_string(),
            "󰀻".to_string(),
            false,
            vec![
                slice("chmod_x", "Make +x", "Grant execute permissions", "󰒓", "tools", false),
                slice("run_binary", "Run App", "Launch application directly", "󰐊", "tools", false),
                slice("sha256_sum", "SHA-256", "Verify SHA-256 signature", "󰒃", "dev", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Send executable via LocalSend", "󰄙", "share", false),
                slice("archive_zst", "To .tar.zst", "Compress binary with Zstandard", "󰀼", "convert", false),
                slice("open_terminal", "Terminal Here", "Open terminal in this directory", "󰆍", "dev", false),
            ],
        ),

        // 13. Generic fallback
        _ => {
            let label = if name_lower.ends_with(".tar.zst")
                || name_lower.ends_with(".tar.xz")
                || name_lower.ends_with(".tar.gz")
            {
                "TAR ARCHIVE".to_string()
            } else if ext.is_empty() {
                "FILE".to_string()
            } else {
                format!("FILE {}", ext.to_uppercase())
            };
            (
                label,
                "󰈔".to_string(),
                false,
                vec![
                    slice("archive_zip", "To .ZIP", "Package into ZIP archive", "󰗄", "convert", false),
                    slice("archive_zst", "To .tar.zst", "Compress with Zstandard", "󰀼", "convert", false),
                    normalize_name_submenu(),
                    slice("sha256_sum", "SHA-256", "Copy SHA-256 checksum to clipboard", "󰒃", "dev", false),
                    share_submenu(),
                    slice("send_localsend", "LocalSend", "Send file via LocalSend", "󰄙", "share", false),
                    slice("copy_data_uri", "Base64", "Copy Base64 encoded string", "󰘦", "dev", false),
                    slice("open_nvim", "In Neovim", "Open file in Neovim", "", "dev", false),
                ],
            )
        }
    };

    PiePayload {
        file_path,
        file_name,
        file_size,
        file_kind,
        file_icon,
        is_image,
        dimensions,
        cursor_x,
        cursor_y,
        monitor_width,
        monitor_height,
        slices,
    }
}

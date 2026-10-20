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
        "Compartir ▸",
        "Enviar por LocalSend, QR Wi-Fi o enlace en la nube",
        "󰐲",
        "share",
        vec![
            slice(
                "send_localsend",
                "LocalSend",
                "Enviar a dispositivos cercanos con LocalSend",
                "󰄙",
                "share",
                false,
            ),
            slice(
                "share_qr",
                "QR Wi-Fi",
                "Servidor HTTP local con código QR instantáneo",
                "󰐲",
                "share",
                true,
            ),
            slice(
                "upload_0x0",
                "Subir 0x0.st",
                "Subir a la nube y generar enlace + QR",
                "󰇚",
                "share",
                true,
            ),
            slice(
                "copy_path",
                "Copiar Ruta",
                "Copiar ruta absoluta al portapapeles",
                "󰅍",
                "dev",
                false,
            ),
            slice(
                "copy_data_uri",
                "Copiar Base64",
                "Codificar archivo como Data URI / Base64",
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
        "Normalizar ▸",
        "Renombrar con formato limpio (kebab, snake, fecha…)",
        "󰑕",
        "dev",
        vec![
            slice(
                "norm_kebab",
                "kebab-case",
                "mi-archivo-final.ext (sin tildes ni espacios)",
                "󰑕",
                "dev",
                false,
            ),
            slice(
                "norm_snake",
                "snake_case",
                "mi_archivo_final.ext (guiones bajos)",
                "󰌌",
                "dev",
                false,
            ),
            slice(
                "norm_camel",
                "camelCase",
                "miArchivoFinal.ext (estilo código)",
                "󰅩",
                "dev",
                false,
            ),
            slice(
                "norm_date",
                "Fecha + Nombre",
                "Añadir prefijo YYYY-MM-DD_ al archivo",
                "󰃭",
                "dev",
                false,
            ),
            slice(
                "norm_clean",
                "Limpiar Basura",
                "Quitar (1), Copy, WhatsApp, paréntesis",
                "󰃢",
                "tools",
                false,
            ),
            slice(
                "norm_lower",
                "minúsculas",
                "Convertir todo el nombre a minúsculas",
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
        "Escalar ▸",
        "Cambiar dimensiones o resolución de la imagen",
        "󰩨",
        "convert",
        vec![
            slice(
                "scale_50",
                "Al 50%",
                "Reducir ancho y alto a la mitad",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_75",
                "Al 75%",
                "Reducción suave manteniendo nitidez",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_25",
                "Al 25%",
                "Miniatura ligera (1/4 de tamaño)",
                "󰩨",
                "convert",
                false,
            ),
            slice(
                "scale_1080",
                "Máx 1080p",
                "Ajustar lado mayor a 1920px (Full HD)",
                "󰍹",
                "convert",
                false,
            ),
            slice(
                "scale_720",
                "Máx 720p",
                "Ajustar lado mayor a 1280px (HD)",
                "󰍹",
                "convert",
                false,
            ),
            slice(
                "scale_square",
                "Avatar 512×512",
                "Recorte cuadrado centrado 1:1",
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
        subs.push(slice("to_webp", "A WebP", "Formato moderno ultra ligero", "󰋩", "convert", false));
    }
    if ext != "png" {
        subs.push(slice("to_png", "A PNG", "Formato sin pérdida con transparencia", "󰸭", "convert", false));
    }
    if ext != "jpg" && ext != "jpeg" {
        subs.push(slice("to_jpg", "A JPG", "Formato JPEG compatible universal", "󰈥", "convert", false));
    }
    if ext != "avif" {
        subs.push(slice("to_avif", "A AVIF", "Compresión de última generación", "󰋩", "convert", false));
    }
    subs.push(slice("to_ico", "A Favicon .ICO", "Icono 64×64 para web/apps", "󰀻", "convert", false));
    subs.push(slice("img_grayscale", "Blanco y Negro", "Convertir a escala de grises", "󰏘", "tools", false));

    menu_slice(
        "menu_convert_img",
        "Convertir ▸",
        "Convertir a WebP, PNG, JPG, AVIF, ICO o B/N",
        "󰋩",
        "convert",
        subs,
    )
}

fn image_transform_submenu() -> PieSlice {
    menu_slice(
        "menu_transform_img",
        "Transformar ▸",
        "Rotar, voltear, limpiar EXIF o extraer colores",
        "󰑵",
        "tools",
        vec![
            slice("rot_90_cw", "Rotar 90° ↻", "Girar 90 grados a la derecha", "󰑵", "tools", false),
            slice("rot_90_ccw", "Rotar 90° ↺", "Girar 90 grados a la izquierda", "󰑴", "tools", false),
            slice("rot_180", "Rotar 180°", "Invertir orientación verticalmente", "󰑵", "tools", false),
            slice("flip_h", "Espejo Horiz.", "Voltear horizontalmente", "󰹳", "tools", false),
            slice("extract_palette", "Paleta HEX", "Extraer 5 colores dominantes al clipboard", "󰏘", "dev", false),
            slice("strip_exif", "Borrar EXIF", "Eliminar GPS y metadatos de cámara", "󰒃", "privacy", false),
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
        .unwrap_or("archivo")
        .to_string();

    let metadata = fs::metadata(path).ok();
    let file_size = metadata
        .as_ref()
        .map(|m| {
            if m.is_dir() {
                let count = fs::read_dir(path).map(|d| d.count()).unwrap_or(0);
                format!("{} elementos", count)
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
            file_kind: "CARPETA".to_string(),
            file_icon: "󰉋".to_string(),
            is_image: false,
            dimensions: String::new(),
            cursor_x,
            cursor_y,
            monitor_width,
            monitor_height,
            slices: vec![
                slice("archive_zip", "Crear .ZIP", "Comprimir carpeta en .zip", "󰗄", "convert", false),
                slice("archive_zst", "Crear .zst", "Comprimir en .tar.zst ultra rápido", "󰀼", "convert", false),
                normalize_name_submenu(),
                slice("send_localsend", "LocalSend", "Enviar carpeta a dispositivo cercano", "󰄙", "share", false),
                share_submenu(),
                slice("open_terminal", "Terminal Aquí", "Abrir terminal en esta carpeta", "󰆍", "dev", false),
                slice("open_nvim", "En Neovim", "Abrir carpeta como proyecto en Neovim", "", "dev", false),
                slice("sha256_sum", "Peso Total", "Calcular uso de disco total", "󰋊", "tools", false),
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
            format!("IMAGEN {}", ext.to_uppercase()),
            "󰋩".to_string(),
            ext != "avif",
            vec![
                image_convert_submenu(&ext),
                image_scale_submenu(),
                slice(
                    "compress_img",
                    "Comprimir Peso",
                    "Reducir MB manteniendo la misma resolución",
                    "󰗄",
                    "convert",
                    false,
                ),
                image_transform_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar imagen con LocalSend", "󰄙", "share", false),
                slice("ocr_text", "OCR Texto", "Extraer texto con Tesseract", "󰗊", "tools", false),
                normalize_name_submenu(),
            ],
        ),

        // 2. Vector SVG
        "svg" => (
            "VECTOR SVG".to_string(),
            "󰜡".to_string(),
            true,
            vec![
                slice("to_png", "Render PNG", "Exportar vector SVG a PNG", "󰸭", "convert", false),
                slice("to_webp", "Render WebP", "Exportar vector SVG a WebP", "󰋩", "convert", false),
                slice("copy_data_uri", "Data URI SVG", "Copiar data:image/svg+xml;base64", "󰅍", "dev", false),
                slice("copy_content", "Copiar XML", "Copiar código fuente del SVG", "󰘦", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar SVG con LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("open_nvim", "Editar SVG", "Abrir código SVG en Neovim", "", "dev", false),
            ],
        ),

        // 3. Video files
        "mp4" | "mkv" | "mov" | "webm" | "avi" | "flv" | "m4v" => (
            format!("VIDEO {}", ext.to_uppercase()),
            "󰕧".to_string(),
            false,
            vec![
                slice("video_to_gif", "A GIF", "Convertir a GIF animado con paleta", "󰵸", "convert", false),
                slice("extract_mp3", "Extraer MP3", "Extraer pista de audio en MP3", "󰎆", "convert", false),
                slice("compress_discord", "Comprimir", "Reducir peso en H.264 (<720p)", "󰙯", "convert", false),
                slice("mute_video", "Silenciar", "Quitar audio sin recodificar", "󰝟", "tools", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar video con LocalSend", "󰄙", "share", false),
                slice("video_thumbnail", "Capturar Frame", "Extraer fotograma en PNG", "󰄀", "tools", false),
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
                    slice("audio_to_opus", "A Opus", "Comprimir a Opus ligero", "󰋋", "convert", false)
                } else {
                    slice("extract_mp3", "A MP3", "Convertir a MP3 192kbps", "󰎆", "convert", false)
                },
                if ext == "wav" {
                    slice("audio_to_flac", "A FLAC", "Comprimir sin pérdida a FLAC", "󰕾", "convert", false)
                } else {
                    slice("audio_to_wav", "A WAV", "Convertir a WAV PCM sin pérdida", "󰕾", "convert", false)
                },
                slice("audio_normalize", "Normalizar Vol", "Nivelar volumen de audio (loudnorm)", "󰓃", "tools", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar audio con LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Copiar checksum SHA-256", "󰒃", "dev", false),
                slice("copy_path", "Copiar Ruta", "Copiar ruta absoluta", "󰅍", "dev", false),
            ],
        ),

        // 5. PDF Documents
        "pdf" => (
            "DOCUMENTO PDF".to_string(),
            "󰈦".to_string(),
            false,
            vec![
                slice("pdf_to_png", "A Imágenes", "Exportar páginas del PDF a PNG", "󰸭", "convert", false),
                slice("archive_zip", "A .ZIP", "Empaquetar PDF en ZIP", "󰗄", "convert", false),
                slice("archive_zst", "A .tar.zst", "Comprimir con Zstandard", "󰀼", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar PDF con LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Calcular firma SHA-256", "󰒃", "dev", false),
                slice("copy_path", "Copiar Ruta", "Copiar ruta del PDF", "󰅍", "dev", false),
            ],
        ),

        // 6. CSV / TSV Data tables
        "csv" | "tsv" => (
            format!("DATOS {}", ext.to_uppercase()),
            "󰈛".to_string(),
            false,
            vec![
                slice("csv_to_json", "A JSON", "Convertir tabla CSV a archivo JSON", "󰘦", "convert", false),
                slice("file_stats", "Contar Filas", "Ver total de registros y columnas", "󰄨", "tools", false),
                slice("archive_zst", "Comprimir .zst", "Comprimir dataset con Zstandard", "󰀼", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar dataset con LocalSend", "󰄙", "share", false),
                slice("open_nvim", "En Neovim", "Inspeccionar datos en Neovim", "", "dev", false),
                slice("sha256_sum", "SHA-256", "Calcular hash de integridad", "󰒃", "dev", false),
            ],
        ),

        // 7. JSON / TOML / YAML Configs
        "json" | "jsonc" | "toml" | "yaml" | "yml" => (
            format!("CONFIG {}", ext.to_uppercase()),
            "󰘦".to_string(),
            false,
            vec![
                if ext.starts_with("json") {
                    slice("json_pretty", "Formatear", "Embellecer e indentar JSON", "󰉼", "tools", false)
                } else {
                    slice("file_stats", "Contar Líneas", "Estadísticas de líneas y bytes", "󰈙", "tools", false)
                },
                if ext.starts_with("json") {
                    slice("json_minify", "Minificar", "Compactar JSON en una línea", "󰗄", "convert", false)
                } else {
                    slice("archive_zip", "A .ZIP", "Empaquetar en archivo ZIP", "󰗄", "convert", false)
                },
                slice("copy_content", "Copiar Todo", "Copiar contenido al portapapeles", "󰅍", "dev", false),
                slice("open_nvim", "En Neovim", "Editar configuración en Neovim", "", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar archivo con LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("copy_data_uri", "Base64", "Copiar codificado en Base64", "󰨿", "dev", false),
            ],
        ),

        // 8. Markdown & Text
        "md" | "txt" | "rst" | "log" => (
            format!("TEXTO {}", ext.to_uppercase()),
            "󰍔".to_string(),
            false,
            vec![
                if ext == "md" {
                    slice("md_to_html", "A HTML", "Exportar Markdown a página HTML", "󰌝", "convert", false)
                } else {
                    slice("archive_zst", "A .tar.zst", "Comprimir texto con Zstandard", "󰀼", "convert", false)
                },
                slice("copy_content", "Copiar Texto", "Copiar todo el contenido al clipboard", "󰅍", "dev", false),
                slice("file_stats", "Contar Palabras", "Ver líneas, palabras y caracteres", "󰈙", "tools", false),
                slice("open_nvim", "En Neovim", "Editar documento en Neovim", "", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar documento con LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("archive_zip", "A .ZIP", "Comprimir documento en ZIP", "󰗄", "convert", false),
            ],
        ),

        // 9. Scripts & Source Code
        "sh" | "bash" | "zsh" | "py" | "rs" | "go" | "js" | "ts" | "lua" | "qml" | "c" | "cpp" | "html" | "css" => (
            format!("CÓDIGO {}", ext.to_uppercase()),
            "󰅩".to_string(),
            false,
            vec![
                slice("open_nvim", "En Neovim", "Abrir código en Neovim", "", "dev", false),
                slice("chmod_x", "Hacer +x", "Dar permisos de ejecución (chmod +x)", "󰒓", "tools", false),
                slice("run_script", "Ejecutar", "Correr script en una terminal nueva", "󰆍", "dev", false),
                slice("copy_content", "Copiar Código", "Copiar código fuente al portapapeles", "󰅍", "dev", false),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar código con LocalSend", "󰄙", "share", false),
                normalize_name_submenu(),
                slice("file_stats", "Líneas (LOC)", "Contar líneas de código", "󰈙", "tools", false),
            ],
        ),

        // 10. Blender / 3D Files
        "blend" | "blend1" | "obj" | "fbx" | "stl" | "gltf" | "glb" => (
            format!("3D {}", ext.to_uppercase()),
            "󰆧".to_string(),
            false,
            vec![
                slice("open_blender", "Abrir Blender", "Abrir escena en Blender", "󰂫", "tools", false),
                slice("archive_zst", "A .tar.zst", "Comprimir proyecto 3D con Zstandard", "󰀼", "convert", false),
                slice("archive_zip", "A .ZIP", "Empaquetar proyecto 3D en ZIP", "󰗄", "convert", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar modelo 3D con LocalSend", "󰄙", "share", false),
                slice("sha256_sum", "SHA-256", "Calcular hash del modelo", "󰒃", "dev", false),
                slice("open_terminal", "Terminal Aquí", "Abrir terminal en esta carpeta", "󰆍", "dev", false),
            ],
        ),

        // 11. Archives & Compressed Packages
        "zip" | "tar" | "gz" | "zst" | "xz" | "bz2" | "7z" | "rar" | "tgz" => (
            format!("PAQUETE {}", ext.to_uppercase()),
            "󰀼".to_string(),
            false,
            vec![
                slice("extract_archive", "Extraer Aquí", "Descomprimir en subcarpeta limpia", "󰿺", "tools", false),
                slice("list_archive", "Ver Archivos", "Contar elementos dentro del paquete", "󰈙", "tools", false),
                slice("sha256_sum", "SHA-256", "Verificar checksum SHA-256", "󰒃", "dev", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar paquete con LocalSend", "󰄙", "share", false),
                slice("copy_path", "Copiar Ruta", "Copiar ruta absoluta al portapapeles", "󰅍", "dev", false),
                slice("open_terminal", "Terminal Aquí", "Abrir terminal en esta carpeta", "󰆍", "dev", false),
            ],
        ),

        // 12. AppImage / Binaries
        "appimage" | "run" | "bin" => (
            "EJECUTABLE".to_string(),
            "󰀻".to_string(),
            false,
            vec![
                slice("chmod_x", "Hacer +x", "Dar permisos de ejecución", "󰒓", "tools", false),
                slice("run_binary", "Ejecutar", "Lanzar aplicación directamente", "󰐊", "tools", false),
                slice("sha256_sum", "SHA-256", "Verificar firma SHA-256", "󰒃", "dev", false),
                normalize_name_submenu(),
                share_submenu(),
                slice("send_localsend", "LocalSend", "Enviar ejecutable con LocalSend", "󰄙", "share", false),
                slice("archive_zst", "A .tar.zst", "Comprimir binario con Zstandard", "󰀼", "convert", false),
                slice("open_terminal", "Terminal Aquí", "Abrir terminal en esta carpeta", "󰆍", "dev", false),
            ],
        ),

        // 13. Generic fallback
        _ => {
            let label = if name_lower.ends_with(".tar.zst")
                || name_lower.ends_with(".tar.xz")
                || name_lower.ends_with(".tar.gz")
            {
                "PAQUETE TAR".to_string()
            } else if ext.is_empty() {
                "ARCHIVO".to_string()
            } else {
                format!("ARCHIVO {}", ext.to_uppercase())
            };
            (
                label,
                "󰈔".to_string(),
                false,
                vec![
                    slice("archive_zip", "A .ZIP", "Empaquetar en archivo ZIP", "󰗄", "convert", false),
                    slice("archive_zst", "A .tar.zst", "Comprimir con Zstandard", "󰀼", "convert", false),
                    normalize_name_submenu(),
                    slice("sha256_sum", "SHA-256", "Copiar firma SHA-256 al portapapeles", "󰒃", "dev", false),
                    share_submenu(),
                    slice("send_localsend", "LocalSend", "Enviar archivo con LocalSend", "󰄙", "share", false),
                    slice("copy_data_uri", "Base64", "Copiar codificado en Base64", "󰘦", "dev", false),
                    slice("open_nvim", "En Neovim", "Abrir archivo en Neovim", "", "dev", false),
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

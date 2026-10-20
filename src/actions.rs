use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};
use qrcode::render::svg;
use qrcode::QrCode;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, UdpSocket};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn notify(title: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "Omarchy Pie", "-i", "document-properties", title, body])
        .spawn();
}

pub fn wl_copy(text: &str) {
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

fn unique_output_path(base_dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let candidate = base_dir.join(format!("{}.{}", stem, ext));
    if !candidate.exists() {
        return candidate;
    }
    for i in 1..1000 {
        let numbered = base_dir.join(format!("{}_{}.{}", stem, i, ext));
        if !numbered.exists() {
            return numbered;
        }
    }
    base_dir.join(format!("{}_out.{}", stem, ext))
}

pub fn execute_action(action_id: &str, file_path: &Path) -> Result<(), String> {
    if !file_path.exists() {
        let msg = format!("File does not exist: {}", file_path.display());
        notify("Error", &msg);
        return Err(msg);
    }

    let parent = file_path.parent().unwrap_or_else(|| Path::new("/tmp"));
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    match action_id {
        // Image format conversions
        "to_webp" => convert_image(file_path, parent, stem, "webp"),
        "to_png" => convert_image(file_path, parent, stem, "png"),
        "to_jpg" => convert_image(file_path, parent, stem, "jpg"),
        "to_ico" => convert_to_ico(file_path, parent, stem),
        "to_avif" => convert_ffmpeg(
            file_path,
            parent,
            stem,
            "avif",
            &["-c:v", "libaom-av1", "-still-picture", "1", "-crf", "30"],
        ),

        // Image compression (same resolution, lower file weight)
        "compress_img" => compress_image_weight(file_path, parent, stem),

        // Image scaling options
        "resize_50" | "scale_50" => scale_image_percent(file_path, parent, stem, 50),
        "scale_75" => scale_image_percent(file_path, parent, stem, 75),
        "scale_25" => scale_image_percent(file_path, parent, stem, 25),
        "scale_1080" => scale_image_max_dim(file_path, parent, stem, 1920, "1080p"),
        "scale_720" => scale_image_max_dim(file_path, parent, stem, 1280, "720p"),
        "scale_square" => scale_image_square(file_path, parent, stem, 512),

        // Image transforms & color tools
        "img_grayscale" => transform_image(file_path, parent, stem, "grayscale"),
        "rot_90_cw" => transform_image(file_path, parent, stem, "rot90"),
        "rot_90_ccw" => transform_image(file_path, parent, stem, "rot270"),
        "rot_180" => transform_image(file_path, parent, stem, "rot180"),
        "flip_h" => transform_image(file_path, parent, stem, "fliph"),
        "extract_palette" => extract_dominant_palette(file_path),
        "strip_exif" => strip_exif_metadata(file_path, parent, stem),
        "ocr_text" => run_ocr(file_path),

        // Filename normalization suite
        "slugify_name" | "norm_kebab" => normalize_filename(file_path, "kebab"),
        "norm_snake" => normalize_filename(file_path, "snake"),
        "norm_camel" => normalize_filename(file_path, "camel"),
        "norm_date" => normalize_filename(file_path, "date"),
        "norm_clean" => normalize_filename(file_path, "clean"),
        "norm_lower" => normalize_filename(file_path, "lower"),

        // Video & Audio actions
        "video_to_gif" => convert_video_to_gif(file_path, parent, stem),
        "extract_mp3" => convert_ffmpeg(
            file_path,
            parent,
            stem,
            "mp3",
            &["-vn", "-c:a", "libmp3lame", "-b:a", "192k"],
        ),
        "audio_to_wav" => convert_ffmpeg(file_path, parent, stem, "wav", &["-vn"]),
        "audio_to_flac" => convert_ffmpeg(file_path, parent, stem, "flac", &["-vn", "-c:a", "flac"]),
        "audio_to_opus" => convert_ffmpeg(
            file_path,
            parent,
            stem,
            "opus",
            &["-vn", "-c:a", "libopus", "-b:a", "96k"],
        ),
        "audio_normalize" => normalize_audio(file_path, parent, stem),
        "compress_discord" => compress_video(file_path, parent, stem),
        "mute_video" => mute_video(file_path, parent, stem),
        "video_thumbnail" => extract_video_frame(file_path, parent, stem),
        "video_to_mp4" => convert_ffmpeg(
            file_path,
            parent,
            stem,
            "mp4",
            &["-c:v", "libx264", "-preset", "fast", "-c:a", "aac"],
        ),
        "video_to_webm" => convert_ffmpeg(
            file_path,
            parent,
            stem,
            "webm",
            &["-c:v", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-c:a", "libopus"],
        ),

        // Documents, Data, Configs & Code
        "pdf_to_png" => convert_pdf_to_png(file_path, parent, stem),
        "csv_to_json" => convert_csv_to_json(file_path, parent, stem),
        "file_stats" => compute_file_stats(file_path),
        "json_pretty" => format_json_file(file_path, parent, stem, true),
        "json_minify" => format_json_file(file_path, parent, stem, false),
        "md_to_html" => convert_md_to_html(file_path, parent, stem),
        "copy_content" => copy_text_content(file_path),
        "chmod_x" => make_executable(file_path),
        "run_script" => run_script_in_terminal(file_path),
        "run_binary" => run_binary_direct(file_path),
        "open_blender" => open_in_blender(file_path),

        // Archives
        "archive_zip" => create_zip(file_path, parent, stem),
        "archive_zst" => create_tar_zst(file_path, parent, stem),
        "extract_archive" => extract_archive(file_path, parent, stem),
        "list_archive" => list_archive_contents(file_path),

        // Universal Dev & Share
        "send_localsend" => send_via_localsend(file_path),
        "copy_data_uri" => copy_as_data_uri(file_path),
        "copy_path" => {
            let s = file_path.to_string_lossy().to_string();
            wl_copy(&s);
            notify("Path copied", &s);
            Ok(())
        }
        "sha256_sum" => compute_sha256_or_dir_size(file_path),
        "open_terminal" => open_in_terminal(file_path),
        "open_nvim" => open_in_nvim(file_path),
        "share_qr" => serve_file_qr(file_path),
        "upload_0x0" => upload_to_0x0(file_path),
        other => {
            let msg = format!("Unknown action: {}", other);
            notify("Omarchy Pie", &msg);
            Err(msg)
        }
    }
}

fn send_via_localsend(input: &Path) -> Result<(), String> {
    let path_str = input.to_string_lossy().to_string();
    notify(
        "Sending via LocalSend…",
        &input.file_name().unwrap_or_default().to_string_lossy(),
    );

    // Use Omarchy's systemd-run pattern for LocalSend headless send
    let status = Command::new("systemd-run")
        .args([
            "--user",
            "--quiet",
            "--collect",
            "localsend",
            "--headless",
            "send",
            &path_str,
        ])
        .status();

    if status.map(|s| s.success()).unwrap_or(false) {
        return Ok(());
    }

    // Fallback to direct launch via uwsm-app
    let _ = Command::new("uwsm-app")
        .args(["--", "localsend", &path_str])
        .spawn()
        .map_err(|e| format!("Could not start LocalSend: {}", e))?;

    Ok(())
}

fn convert_image(input: &Path, parent: &Path, stem: &str, target_ext: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, target_ext);
    notify(
        "Converting image…",
        &format!(
            "{} ➔ {}",
            input.file_name().unwrap_or_default().to_string_lossy(),
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );

    let in_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if in_ext != "svg" {
        if let Ok(img) = image::open(input) {
            let fmt = match target_ext {
                "png" => ImageFormat::Png,
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "webp" => ImageFormat::WebP,
                _ => ImageFormat::Png,
            };
            if img.save_with_format(&output, fmt).is_ok() {
                let out_str = output.to_string_lossy().to_string();
                wl_copy(&out_str);
                notify(
                    "Conversion completed",
                    &format!(
                        "Saved to {}\n(Path copied to clipboard)",
                        output.file_name().unwrap_or_default().to_string_lossy()
                    ),
                );
                return Ok(());
            }
        }
    }

    convert_ffmpeg(input, parent, stem, target_ext, &[])
}

fn convert_to_ico(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "ico");
    let img = image::open(input).map_err(|e| format!("Could not open image: {}", e))?;
    let icon = img.resize_exact(64, 64, FilterType::Lanczos3);
    icon.save_with_format(&output, ImageFormat::Ico)
        .map_err(|e| format!("Error saving ICO: {}", e))?;
    wl_copy(&output.to_string_lossy());
    notify(
        "Favicon .ICO created (64×64)",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn compress_image_weight(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let orig_size = fs::metadata(input).map(|m| m.len()).unwrap_or(1);
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // Keep exact resolution, optimize compression via FFmpeg / WebP / JPEG quality 76
    let out_ext = if ext == "png" { "webp" } else { &ext };
    let output = unique_output_path(parent, &format!("{}_opt", stem), out_ext);

    let extra: Vec<&str> = match out_ext {
        "jpg" | "jpeg" => vec!["-q:v", "6"],
        "webp" => vec!["-quality", "78"],
        _ => vec![],
    };

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-i", input.to_str().unwrap_or("")]);
    cmd.args(&extra);
    cmd.arg(output.to_str().unwrap_or(""));

    let ok = cmd.output().map(|o| o.status.success()).unwrap_or(false);
    if !ok {
        let img = image::open(input).map_err(|e| e.to_string())?;
        img.save(&output).map_err(|e| e.to_string())?;
    }

    let new_size = fs::metadata(&output).map(|m| m.len()).unwrap_or(orig_size);
    let saved_pct = if orig_size > new_size {
        ((orig_size - new_size) as f64 / orig_size as f64 * 100.0).round() as i32
    } else {
        0
    };

    wl_copy(&output.to_string_lossy());
    notify(
        "Image compressed (same resolution)",
        &format!(
            "{} ➔ {} (-{}%)\n{}",
            crate::context::format_size(orig_size),
            crate::context::format_size(new_size),
            saved_pct,
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn scale_image_percent(
    input: &Path,
    parent: &Path,
    stem: &str,
    pct: u32,
) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let output = unique_output_path(parent, &format!("{}_{}pct", stem, pct), &ext);

    let img = image::open(input).map_err(|e| format!("Could not open image: {}", e))?;
    let new_w = ((img.width() * pct) / 100).max(1);
    let new_h = ((img.height() * pct) / 100).max(1);
    let resized = img.resize_exact(new_w, new_h, FilterType::Lanczos3);
    resized
        .save(&output)
        .map_err(|e| format!("Error saving image: {}", e))?;

    wl_copy(&output.to_string_lossy());
    notify(
        &format!("Image scaled to {}%", pct),
        &format!(
            "{}×{} ➔ {}×{} ({})",
            img.width(),
            img.height(),
            new_w,
            new_h,
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn scale_image_max_dim(
    input: &Path,
    parent: &Path,
    stem: &str,
    max_dim: u32,
    tag: &str,
) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let output = unique_output_path(parent, &format!("{}_{}", stem, tag), &ext);

    let img = image::open(input).map_err(|e| format!("Could not open image: {}", e))?;
    let resized = img.resize(max_dim, max_dim, FilterType::Lanczos3);
    resized
        .save(&output)
        .map_err(|e| format!("Error saving image: {}", e))?;

    wl_copy(&output.to_string_lossy());
    notify(
        &format!("Image fitted to {}", tag),
        &format!(
            "{}×{} ➔ {}",
            resized.width(),
            resized.height(),
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn scale_image_square(
    input: &Path,
    parent: &Path,
    stem: &str,
    side: u32,
) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let output = unique_output_path(parent, &format!("{}_{}x{}", stem, side, side), &ext);

    let mut img = image::open(input).map_err(|e| format!("Could not open image: {}", e))?;
    let (w, h) = img.dimensions();
    let min_side = w.min(h);
    let x = (w - min_side) / 2;
    let y = (h - min_side) / 2;
    let cropped = img.crop(x, y, min_side, min_side);
    let squared = cropped.resize_exact(side, side, FilterType::Lanczos3);
    squared
        .save(&output)
        .map_err(|e| format!("Error saving avatar: {}", e))?;

    wl_copy(&output.to_string_lossy());
    notify(
        &format!("Avatar {}×{} created", side, side),
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn transform_image(input: &Path, parent: &Path, stem: &str, op: &str) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let output = unique_output_path(parent, &format!("{}_{}", stem, op), &ext);

    let img = image::open(input).map_err(|e| format!("Error opening image: {}", e))?;
    let out_img = match op {
        "grayscale" => img.grayscale(),
        "rot90" => img.rotate90(),
        "rot270" => img.rotate270(),
        "rot180" => img.rotate180(),
        "fliph" => img.fliph(),
        _ => img,
    };

    out_img
        .save(&output)
        .map_err(|e| format!("Error saving image: {}", e))?;
    wl_copy(&output.to_string_lossy());
    notify(
        "Transformation completed",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn extract_dominant_palette(input: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|e| format!("Could not open image: {}", e))?;
    let small = img.resize(64, 64, FilterType::Nearest).to_rgba8();
    let mut counts: HashMap<(u8, u8, u8), usize> = HashMap::new();

    for pixel in small.pixels() {
        if pixel[3] < 128 {
            continue;
        }
        // Quantize to 24-step buckets so similar shades group together
        let r = (pixel[0] / 24) * 24;
        let g = (pixel[1] / 24) * 24;
        let b = (pixel[2] / 24) * 24;
        *counts.entry((r, g, b)).or_insert(0) += 1;
    }

    let mut sorted: Vec<((u8, u8, u8), usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let hex_colors: Vec<String> = sorted
        .into_iter()
        .take(5)
        .map(|((r, g, b), _)| format!("#{:02X}{:02X}{:02X}", r, g, b))
        .collect();

    let result = hex_colors.join("  ");
    wl_copy(&result);
    notify("HEX palette copied to clipboard", &result);
    Ok(())
}

fn strip_exif_metadata(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let output = unique_output_path(parent, &format!("{}_clean", stem), &ext);

    let img = image::open(input).map_err(|e| format!("Error reading image: {}", e))?;
    img.save(&output)
        .map_err(|e| format!("Error saving clean image: {}", e))?;

    wl_copy(&output.to_string_lossy());
    notify(
        "EXIF metadata stripped",
        &format!(
            "Clean file: {}",
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn run_ocr(input: &Path) -> Result<(), String> {
    notify("Running OCR…", "Extracting text with Tesseract");
    let output = Command::new("tesseract")
        .args([input.to_str().unwrap_or(""), "stdout"])
        .output()
        .map_err(|e| format!("Error executing tesseract: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        notify("OCR Error", &err);
        return Err(err.to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        notify(
            "OCR: No Results",
            "No legible text was detected in the image.",
        );
        return Ok(());
    }

    wl_copy(&text);
    let preview: String = text.chars().take(90).collect();
    notify(
        "Text extracted to clipboard",
        &format!("\"{}…\" ({} characters)", preview.replace('\n', " "), text.len()),
    );
    Ok(())
}

fn extract_video_frame(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, &format!("{}_frame", stem), "png");
    let res = Command::new("ffmpeg")
        .args([
            "-y",
            "-ss",
            "00:00:01",
            "-i",
            input.to_str().unwrap_or(""),
            "-frames:v",
            "1",
            output.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !res.status.success() {
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                input.to_str().unwrap_or(""),
                "-frames:v",
                "1",
                output.to_str().unwrap_or(""),
            ])
            .status();
    }

    wl_copy(&output.to_string_lossy());
    notify(
        "Frame captured",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn normalize_audio(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3");
    convert_ffmpeg(
        input,
        parent,
        &format!("{}_norm", stem),
        ext,
        &["-af", "loudnorm=I=-16:TP=-1.5:LRA=11"],
    )
}

fn convert_pdf_to_png(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let out_prefix = parent.join(format!("{}_page", stem));
    if let Ok(status) = Command::new("pdftoppm")
        .args([
            "-png",
            "-r",
            "150",
            input.to_str().unwrap_or(""),
            out_prefix.to_str().unwrap_or(""),
        ])
        .status()
    {
        if status.success() {
            notify(
                "PDF exported to PNG",
                &format!("Pages saved as {}_page-*.png", stem),
            );
            return Ok(());
        }
    }
    convert_ffmpeg(
        input,
        parent,
        &format!("{}_page1", stem),
        "png",
        &["-frames:v", "1"],
    )
}

fn split_csv_line(line: &str, delim: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch == delim && !in_quotes {
            fields.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn convert_csv_to_json(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "json");
    let file = File::open(input).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let Some(Ok(header_line)) = lines.next() else {
        return Err("CSV file is empty".to_string());
    };
    let delim = if header_line.contains('\t') && !header_line.contains(',') {
        '\t'
    } else if header_line.contains(';') && !header_line.contains(',') {
        ';'
    } else {
        ','
    };

    let headers = split_csv_line(&header_line, delim);
    let mut rows = Vec::new();

    for line in lines.flatten() {
        if line.trim().is_empty() {
            continue;
        }
        let vals = split_csv_line(&line, delim);
        let mut obj = serde_json::Map::new();
        for (i, h) in headers.iter().enumerate() {
            let key = if h.is_empty() {
                format!("col_{}", i + 1)
            } else {
                h.clone()
            };
            let val_str = vals.get(i).cloned().unwrap_or_default();
            let val = if let Ok(n) = val_str.parse::<i64>() {
                Value::Number(n.into())
            } else if let Ok(f) = val_str.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::String(val_str))
            } else {
                Value::String(val_str)
            };
            obj.insert(key, val);
        }
        rows.push(Value::Object(obj));
    }

    let json_text = serde_json::to_string_pretty(&rows)
        .map_err(|e| format!("Error serializing JSON: {}", e))?;
    fs::write(&output, json_text).map_err(|e| e.to_string())?;
    wl_copy(&output.to_string_lossy());
    notify(
        "CSV converted to JSON",
        &format!(
            "{} rows ➔ {}",
            rows.len(),
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn compute_file_stats(input: &Path) -> Result<(), String> {
    let file = File::open(input).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines_count = 0usize;
    let mut words_count = 0usize;
    let mut cols_count = 0usize;

    for (i, line) in reader.lines().flatten().enumerate() {
        lines_count += 1;
        words_count += line.split_whitespace().count();
        if i == 0 {
            cols_count = line.split(',').count().max(line.split('\t').count());
        }
    }

    let summary = if cols_count > 1 {
        format!(
            "{} rows • {} columns • {} words",
            lines_count, cols_count, words_count
        )
    } else {
        format!("{} lines • {} words", lines_count, words_count)
    };
    wl_copy(&summary);
    notify("File statistics (copied)", &summary);
    Ok(())
}

fn format_json_file(
    input: &Path,
    parent: &Path,
    stem: &str,
    pretty: bool,
) -> Result<(), String> {
    let raw = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON: {}", e))?;

    let formatted = if pretty {
        serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())?
    } else {
        serde_json::to_string(&parsed).map_err(|e| e.to_string())?
    };

    let suffix = if pretty { "pretty" } else { "min" };
    let output = unique_output_path(parent, &format!("{}_{}", stem, suffix), "json");
    fs::write(&output, &formatted).map_err(|e| e.to_string())?;
    wl_copy(&formatted);
    notify(
        if pretty {
            "JSON formatted"
        } else {
            "JSON minified"
        },
        &format!(
            "Saved to {} and copied to clipboard",
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn convert_md_to_html(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "html");
    let md = fs::read_to_string(input).map_err(|e| e.to_string())?;

    let mut body = String::new();
    for line in md.lines() {
        if let Some(h) = line.strip_prefix("### ") {
            body.push_str(&format!("<h3>{}</h3>\n", h));
        } else if let Some(h) = line.strip_prefix("## ") {
            body.push_str(&format!("<h2>{}</h2>\n", h));
        } else if let Some(h) = line.strip_prefix("# ") {
            body.push_str(&format!("<h1>{}</h1>\n", h));
        } else if let Some(li) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            body.push_str(&format!("<li>{}</li>\n", li));
        } else if line.trim().is_empty() {
            body.push_str("<br/>\n");
        } else {
            body.push_str(&format!("<p>{}</p>\n", line));
        }
    }

    let html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<style>body{{font-family:system-ui,-apple-system,sans-serif;max-width:780px;margin:40px auto;padding:0 20px;line-height:1.6;background:#111318;color:#e2e4e9}}h1,h2,h3{{color:#89b4fa}}code,pre{{background:#1e1e2e;padding:2px 6px;border-radius:4px}}</style>\n</head>\n<body>\n{}\n</body>\n</html>",
        stem, body
    );

    fs::write(&output, html).map_err(|e| e.to_string())?;
    wl_copy(&output.to_string_lossy());
    notify(
        "Markdown exported to HTML",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn copy_text_content(input: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| format!("Error reading text: {}", e))?;
    wl_copy(&text);
    notify(
        "Content copied to clipboard",
        &format!("{} characters copied", text.len()),
    );
    Ok(())
}

fn make_executable(input: &Path) -> Result<(), String> {
    let meta = fs::metadata(input).map_err(|e| e.to_string())?;
    let mut perms = meta.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(input, perms).map_err(|e| e.to_string())?;
    notify(
        "Permissions updated",
        &format!(
            "chmod +x applied to {}",
            input.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn run_script_in_terminal(input: &Path) -> Result<(), String> {
    let dir = input.parent().unwrap_or_else(|| Path::new("."));
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let runner = match ext.as_str() {
        "py" => "python3",
        "js" => "node",
        "lua" => "lua",
        _ => "bash",
    };
    let cmd_str = format!(
        "{} '{}'; echo ''; read -p 'Press Enter to close...'",
        runner,
        input.to_string_lossy()
    );
    let _ = Command::new("uwsm-app")
        .args([
            "--",
            "xdg-terminal-exec",
            &format!("--dir={}", dir.to_string_lossy()),
            "-e",
            "bash",
            "-c",
            &cmd_str,
        ])
        .spawn();
    Ok(())
}

fn run_binary_direct(input: &Path) -> Result<(), String> {
    let _ = make_executable(input);
    let _ = Command::new(input).spawn();
    notify(
        "Launching application",
        &input.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn open_in_blender(input: &Path) -> Result<(), String> {
    let _ = Command::new("uwsm-app")
        .args(["--", "blender", input.to_str().unwrap_or("")])
        .spawn();
    notify(
        "Opening in Blender",
        &input.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn list_archive_contents(input: &Path) -> Result<(), String> {
    let name_lower = input.to_string_lossy().to_lowercase();
    let out = if name_lower.ends_with(".zip") {
        Command::new("unzip")
            .args(["-l", input.to_str().unwrap_or("")])
            .output()
    } else {
        Command::new("tar")
            .args(["-tf", input.to_str().unwrap_or("")])
            .output()
    };

    if let Ok(o) = out {
        let text = String::from_utf8_lossy(&o.stdout);
        let count = text.lines().count();
        wl_copy(&text);
        notify(
            "Archive contents (copied)",
            &format!("{} entries listed to clipboard", count),
        );
        return Ok(());
    }
    Err("Could not list archive contents".to_string())
}

fn copy_as_data_uri(input: &Path) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| format!("Could not read file: {}", e))?;
    if bytes.len() > 25 * 1024 * 1024 {
        let msg = "File exceeds 25 MB size limit for clipboard Data URI".to_string();
        notify("File too large", &msg);
        return Err(msg);
    }

    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "svg" => Some("image/svg+xml"),
        "avif" => Some("image/avif"),
        _ => None,
    };

    let b64 = BASE64.encode(&bytes);
    let final_str = if let Some(m) = mime {
        format!("data:{};base64,{}", m, b64)
    } else {
        b64
    };

    wl_copy(&final_str);
    notify(
        if mime.is_some() {
            "Data URI copied"
        } else {
            "Base64 copied"
        },
        &format!("{} characters copied to clipboard", final_str.len()),
    );
    Ok(())
}

fn compute_sha256_or_dir_size(input: &Path) -> Result<(), String> {
    if input.is_dir() {
        let output = Command::new("du")
            .args(["-sh", input.to_str().unwrap_or(".")])
            .output()
            .map_err(|e| e.to_string())?;
        let info = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("0B")
            .to_string();
        wl_copy(&info);
        notify("Folder size", &format!("Total: {} (copied)", info));
        return Ok(());
    }

    let mut file = File::open(input).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let hash = format!("{:x}", hasher.finalize());
    wl_copy(&hash);
    notify(
        "SHA-256 copied to clipboard",
        &format!("{}…{}", &hash[..12], &hash[hash.len() - 12..]),
    );
    Ok(())
}

fn tokenize_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in s.chars() {
        let mapped = match ch {
            'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => Some('a'),
            'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => Some('e'),
            'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => Some('i'),
            'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => Some('o'),
            'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => Some('u'),
            'ñ' | 'Ñ' => Some('n'),
            c if c.is_ascii_alphanumeric() => Some(c),
            _ => None,
        };
        if let Some(c) = mapped {
            // Split camelCase boundary if uppercase follows lowercase
            if c.is_ascii_uppercase()
                && current
                    .chars()
                    .last()
                    .map(|last| last.is_ascii_lowercase())
                    .unwrap_or(false)
            {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(c);
        } else if !current.is_empty() {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
    }
    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }
    words
}

fn slugify_string(s: &str) -> String {
    tokenize_words(s).join("-")
}

fn current_iso_date() -> String {
    if let Ok(out) = Command::new("date").arg("+%Y-%m-%d").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }
    "2026-10-20".to_string()
}

fn normalize_filename(input: &Path, mode: &str) -> Result<(), String> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let words = tokenize_words(stem);
    let new_stem = match mode {
        "kebab" => words.join("-"),
        "snake" => words.join("_"),
        "camel" => {
            let mut out = String::new();
            for (i, w) in words.iter().enumerate() {
                if i == 0 {
                    out.push_str(w);
                } else {
                    let mut chars = w.chars();
                    if let Some(first) = chars.next() {
                        out.push(first.to_ascii_uppercase());
                        out.push_str(chars.as_str());
                    }
                }
            }
            out
        }
        "date" => {
            let base = words.join("-");
            let date = current_iso_date();
            if base.starts_with(&date) {
                base
            } else {
                format!("{}_{}", date, base)
            }
        }
        "clean" => {
            // Strip common clutter words: copy, copia, whatsapp, at, final, duplicate numbers
            let filtered: Vec<String> = words
                .into_iter()
                .filter(|w| {
                    !matches!(
                        w.as_str(),
                        "copy" | "copia" | "of" | "de" | "at" | "the" | "1" | "2" | "3"
                    )
                })
                .collect();
            filtered.join("-")
        }
        "lower" => stem
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        _ => words.join("-"),
    };

    let final_stem = if new_stem.is_empty() {
        "file".to_string()
    } else {
        new_stem
    };

    let new_path = if input.is_dir() {
        parent.join(&final_stem)
    } else if let Some(ext) = input.extension().and_then(|e| e.to_str()) {
        parent.join(format!("{}.{}", final_stem, ext.to_lowercase()))
    } else {
        parent.join(&final_stem)
    };

    if new_path == input {
        notify("Filename normalized", "The file already has that format.");
        return Ok(());
    }
    if new_path.exists() {
        let msg = format!(
            "Already exists: {}",
            new_path.file_name().unwrap_or_default().to_string_lossy()
        );
        notify("Could not rename", &msg);
        return Err(msg);
    }

    fs::rename(input, &new_path).map_err(|e| format!("Error renaming: {}", e))?;
    wl_copy(&new_path.to_string_lossy());
    notify(
        &format!("Renamed ({})", mode),
        &format!(
            "➔ {}",
            new_path.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn convert_ffmpeg(
    input: &Path,
    parent: &Path,
    stem: &str,
    target_ext: &str,
    extra_args: &[&str],
) -> Result<(), String> {
    let output = unique_output_path(parent, stem, target_ext);
    notify(
        "Processing with FFmpeg…",
        &format!(
            "Generating {}",
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-i", input.to_str().unwrap_or("")]);
    cmd.args(extra_args);
    cmd.arg(output.to_str().unwrap_or(""));

    let res = cmd.output().map_err(|e| format!("Error executing ffmpeg: {}", e))?;
    if !res.status.success() {
        let err = String::from_utf8_lossy(&res.stderr);
        notify("Conversion error", "FFmpeg returned an error.");
        return Err(err.to_string());
    }

    wl_copy(&output.to_string_lossy());
    notify(
        "Conversion ready",
        &format!(
            "Saved: {}",
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn convert_video_to_gif(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "gif");
    notify(
        "Generating animated GIF…",
        "Optimizing color palette with FFmpeg",
    );

    let filter = "fps=15,scale=640:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse";
    let res = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input.to_str().unwrap_or(""),
            "-vf",
            filter,
            "-loop",
            "0",
            output.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !res.status.success() {
        notify("Error creating GIF", "Could not convert video to GIF.");
        return Err("ffmpeg gif failed".to_string());
    }

    wl_copy(&output.to_string_lossy());
    notify(
        "GIF created successfully",
        &format!(
            "{} (Path copied to clipboard)",
            output.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    Ok(())
}

fn compress_video(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, &format!("{}_compact", stem), "mp4");
    notify(
        "Compressing video…",
        "Re-encoding to lightweight H.264 for easy sharing",
    );

    let res = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input.to_str().unwrap_or(""),
            "-vf",
            "scale='min(1280,iw)':-2",
            "-c:v",
            "libx264",
            "-crf",
            "28",
            "-preset",
            "veryfast",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            output.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !res.status.success() {
        notify("Error compressing", "FFmpeg failed to compress video.");
        return Err("compress failed".to_string());
    }

    let size_str = fs::metadata(&output)
        .map(|m| crate::context::format_size(m.len()))
        .unwrap_or_default();
    wl_copy(&output.to_string_lossy());
    notify(
        "Compressed video ready",
        &format!(
            "{} ({})",
            output.file_name().unwrap_or_default().to_string_lossy(),
            size_str
        ),
    );
    Ok(())
}

fn mute_video(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4");
    convert_ffmpeg(
        input,
        parent,
        &format!("{}_muted", stem),
        ext,
        &["-c:v", "copy", "-an"],
    )
}

fn create_zip(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "zip");
    let file_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    let res = Command::new("zip")
        .current_dir(parent)
        .args(["-r", "-q", output.to_str().unwrap_or(""), file_name])
        .output()
        .map_err(|e| e.to_string())?;

    if !res.status.success() {
        notify("Error creating ZIP", "The zip command returned an error.");
        return Err("zip failed".to_string());
    }

    wl_copy(&output.to_string_lossy());
    notify(
        "ZIP archive created",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn create_tar_zst(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let output = unique_output_path(parent, stem, "tar.zst");
    let file_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    let res = Command::new("tar")
        .current_dir(parent)
        .args(["--zstd", "-cf", output.to_str().unwrap_or(""), file_name])
        .output()
        .map_err(|e| e.to_string())?;

    if !res.status.success() {
        notify("Error creating .tar.zst", "The tar command returned an error.");
        return Err("tar.zst failed".to_string());
    }

    wl_copy(&output.to_string_lossy());
    notify(
        ".tar.zst package created",
        &output.file_name().unwrap_or_default().to_string_lossy(),
    );
    Ok(())
}

fn extract_archive(input: &Path, parent: &Path, stem: &str) -> Result<(), String> {
    let clean_stem = stem.strip_suffix(".tar").unwrap_or(stem);
    let out_dir = parent.join(format!("{}_extracted", clean_stem));
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let name_lower = input.to_string_lossy().to_lowercase();
    let status = if name_lower.ends_with(".zip") {
        Command::new("unzip")
            .args([
                "-q",
                input.to_str().unwrap_or(""),
                "-d",
                out_dir.to_str().unwrap_or(""),
            ])
            .status()
    } else {
        Command::new("tar")
            .args([
                "-xf",
                input.to_str().unwrap_or(""),
                "-C",
                out_dir.to_str().unwrap_or(""),
            ])
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            wl_copy(&out_dir.to_string_lossy());
            notify(
                "Archive extracted",
                &format!(
                    "Folder: {}",
                    out_dir.file_name().unwrap_or_default().to_string_lossy()
                ),
            );
            Ok(())
        }
        _ => {
            notify("Error extracting", "Could not decompress archive.");
            Err("extract failed".to_string())
        }
    }
}

fn open_in_terminal(input: &Path) -> Result<(), String> {
    let dir = if input.is_dir() {
        input
    } else {
        input.parent().unwrap_or_else(|| Path::new("."))
    };
    let _ = Command::new("uwsm-app")
        .args([
            "--",
            "xdg-terminal-exec",
            &format!("--dir={}", dir.to_string_lossy()),
        ])
        .spawn();
    Ok(())
}

fn open_in_nvim(input: &Path) -> Result<(), String> {
    let dir = if input.is_dir() {
        input
    } else {
        input.parent().unwrap_or_else(|| Path::new("."))
    };
    let _ = Command::new("uwsm-app")
        .args([
            "--",
            "xdg-terminal-exec",
            &format!("--dir={}", dir.to_string_lossy()),
            "-e",
            "nvim",
            input.to_str().unwrap_or("."),
        ])
        .spawn();
    Ok(())
}

fn detect_lan_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn generate_qr_svg(url: &str, out_path: &Path) -> Result<(), String> {
    let code = QrCode::new(url.as_bytes()).map_err(|e| e.to_string())?;
    let svg_xml = code
        .render()
        .min_dimensions(240, 240)
        .dark_color(svg::Color("#111318"))
        .light_color(svg::Color("#ffffff"))
        .build();
    fs::write(out_path, svg_xml).map_err(|e| e.to_string())?;
    Ok(())
}

fn push_qr_to_shell(url: &str, qr_path: &Path, subtitle: &str) {
    let payload = json!({
        "url": url,
        "qrPath": qr_path.to_string_lossy(),
        "subtitle": subtitle
    })
    .to_string();

    let _ = fs::write("/tmp/omarchy-pie/qr_state.json", &payload);

    for id in ["io.github.jeffcortez23.omarchy-pie", "elyefris.pie"] {
        let _ = Command::new("/usr/share/omarchy/bin/omarchy-shell")
            .env("OMARCHY_PATH", "/usr/share/omarchy")
            .args(["shell", "call", id, "showQr", &payload])
            .status();
    }
}

fn upload_to_0x0(input: &Path) -> Result<(), String> {
    let tmp_dir = PathBuf::from("/tmp/omarchy-pie");
    let _ = fs::create_dir_all(&tmp_dir);

    let res = Command::new("curl")
        .args([
            "-sS",
            "-A",
            "omarchy-pie/0.1",
            "-F",
            &format!("file=@{}", input.to_string_lossy()),
            "https://0x0.st",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let url = String::from_utf8_lossy(&res.stdout).trim().to_string();
    if !res.status.success() || !url.starts_with("http") {
        notify(
            "0x0.st unavailable",
            "Falling back to local Wi-Fi QR server",
        );
        return serve_file_qr(input);
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let qr_path = tmp_dir.join(format!("qr_cloud_{}.svg", ts));
    let _ = generate_qr_svg(&url, &qr_path);
    wl_copy(&url);
    push_qr_to_shell(&url, &qr_path, "Public link on 0x0.st (copied)");
    notify("Uploaded to 0x0.st", &format!("{} (copied to clipboard)", url));
    Ok(())
}

fn serve_file_qr(input: &Path) -> Result<(), String> {
    let tmp_dir = PathBuf::from("/tmp/omarchy-pie");
    let _ = fs::create_dir_all(&tmp_dir);

    let pid_file = tmp_dir.join("server.pid");
    if let Ok(old_pid) = fs::read_to_string(&pid_file) {
        let old_pid = old_pid.trim();
        if !old_pid.is_empty() && old_pid != std::process::id().to_string() {
            let _ = Command::new("kill").arg(old_pid).status();
        }
    }
    let _ = fs::write(&pid_file, std::process::id().to_string());

    let serve_target: PathBuf = if input.is_dir() {
        let stem = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("folder");
        let zip_path = tmp_dir.join(format!("{}.zip", stem));
        let _ = fs::remove_file(&zip_path);
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let _ = Command::new("zip")
            .current_dir(parent)
            .args(["-r", "-q", zip_path.to_str().unwrap_or(""), stem])
            .status();
        zip_path
    } else {
        input.to_path_buf()
    };

    let listener = TcpListener::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let lan_ip = detect_lan_ip();
    let file_name = serve_target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download.bin");
    let safe_name = slugify_string(
        serve_target
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file"),
    );
    let ext = serve_target
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let url = format!("http://{}:{}/{}.{}", lan_ip, port, safe_name, ext);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let qr_path = tmp_dir.join(format!("qr_wifi_{}.svg", ts));
    generate_qr_svg(&url, &qr_path)?;

    wl_copy(&url);
    push_qr_to_shell(&url, &qr_path, "Scan with your phone on same Wi-Fi network");

    let _ = listener.set_nonblocking(false);
    let start = SystemTime::now();
    for _ in 0..8 {
        if start.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(600) {
            break;
        }
        if let Ok((mut stream, peer)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);

            if request_line.contains("favicon.ico") {
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                continue;
            }

            if let Ok(mut f) = File::open(&serve_target) {
                let len = f.metadata().map(|m| m.len()).unwrap_or(0);
                let headers = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nConnection: close\r\n\r\n",
                    len, file_name
                );
                if stream.write_all(headers.as_bytes()).is_ok() {
                    if std::io::copy(&mut f, &mut stream).is_ok() {
                        notify(
                            "File downloaded via Wi-Fi",
                            &format!("Sent to {}", peer.ip()),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

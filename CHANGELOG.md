# Changelog

All notable changes to the **Omarchy Pie** plugin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-10-20

### Added
- **Radial Pie Menu Interface (`PieOverlay.qml`)**:
  - Fullscreen GPU-accelerated Wayland overlay implemented in Quickshell (`QQuickPaintedItem` / HTML5 Canvas).
  - Multi-level radial slices with outer glow, dynamic hover physics, smooth ease-out entry animation, and sound feedback.
  - Slices with iconography, descriptive primary labels, and secondary action subtitles.
  - Interactive central file hub displaying the active target filename, mime category badge, file size in KB/MB, and parent folder path.
  - Embedded SVG QR code viewer in center hub for instant mobile scanning.
  - Keyboard navigation: numeric triggers (`1`-`8`), `Esc` / `Backspace` to navigate back or dismiss, and quick slice selection.
  - Sub-menu hierarchy support: breadcrumb trails, return slices (`Atrás`), and slice transitions.
- **High-Performance Rust Backend (`omarchy-pie`)**:
  - Fast file detection using mime inference, magic byte sniffing, and metadata probing.
  - Context engine (`src/context.rs`) generating dynamic slice menus tailored specifically to the targeted file extension.
  - Comprehensive action execution engine (`src/actions.rs`) with automated notifications and clipboard integration.
  - Zero-lag startup (~12ms) with asynchronous worker execution.
- **Image Processing & Modern Conversions**:
  - Direct format conversion to modern WebP, AVIF, PNG, JPEG, and `.ICO` Favicon (64×64 multi-layer).
  - High-quality Lanczos3 image scaling presets: `50%`, `75%`, `25%`, `1080p Full HD`, `720p HD`, and `512×512 Square Avatar`.
  - Quality-controlled file compression (`Comprimir Peso`) optimizing byte size while preserving original dimensions.
  - Image transformations: clockwise/counter-clockwise 90° rotation, 180° rotation, and horizontal mirror flip.
  - 5-color dominant palette extraction directly to clipboard in `#RRGGBB` format.
  - Lossless EXIF and GPS geolocation metadata stripping via ImageMagick/ExifTool.
  - Optical Character Recognition (OCR) powered by Tesseract, capturing text directly to the Wayland clipboard.
- **Sharing & Local Device Transfer**:
  - **LocalSend Integration**: Headless direct device sharing via `localsend --headless send` with automatic GUI fallback.
  - **Ephemeral Local Wi-Fi HTTP Server**: Lightweight internal Rust HTTP server generating a one-time scannable QR code in the radial center for zero-config mobile downloads on local networks.
  - **Cloud Upload (`0x0.st`)**: 1-click temporary hosting upload with direct clipboard URL copy and QR display.
  - **Base64 Data URI**: Direct encoding to `data:<mime>;base64,...` formatted strings for web developers.
- **Intelligent Filename Normalization**:
  - Safe rename presets: `kebab-case`, `snake_case`, `camelCase`, and `lowercase`.
  - ISO timestamp prefixing (`YYYY-MM-DD_filename`) for structured document archiving.
  - Clutter cleaner: removes unwanted suffixes and noise (e.g., `(1)`, `Copy of`, `_final`, leading/trailing spaces).
- **Video & Audio Processing**:
  - Video to optimized 50-color palette animated GIF via `ffmpeg`.
  - High-fidelity audio extraction (MP3 192kbps).
  - Fast video size compression (H.264 CRF 28, 720p maximum bound).
  - Audio muting / stream stripping from video files.
  - Video keyframe snapshot extraction to high-resolution PNG.
  - Audio loudness normalization adhering to EBU R128 (`loudnorm`) broadcasting standards.
  - Direct transcode options for WebM, MP4, Opus, and lossless FLAC.
- **Developer, Data & Document Utilities**:
  - CSV to JSON converter with automatic type inference and row/column count diagnostics.
  - JSON pretty-formatter and single-line minifier.
  - Markdown to modern styled HTML converter with GitHub-style typography and dark-mode styling.
  - PDF document exporter: extracts all pages into high-resolution PNG images.
  - Script utilities: `chmod +x` executable toggling and 1-click execution in default terminal emulator.
  - Quick launcher to open target in Neovim or Blender.
  - Archive unpacking and listing (`.zip`, `.tar.gz`, `.tar.xz`, `.tar.zst`).
  - SHA-256 cryptographic checksum calculation with instant clipboard copy.
- **Omarchy Shell Integration**:
  - Seamless background daemon service (`Service.qml`) managing IPC signals.
  - Minimalist bar widget (`BarWidget.qml`) with status icon, hover glow, and quick launch trigger.
  - Automatic `~/.local/bin/omarchy-pie` symlink creation on plugin activation.
  - Nautilus file manager right-click script integration (`~/.local/share/nautilus/scripts/Omarchy Pie`).
  - Compositor-native Hyprland shortcut binding (`SUPER + Z`) with clean selection capture.
  - Native Omarchy palette adaptation, dynamically inheriting active desktop theme colors, fonts, and border radii.

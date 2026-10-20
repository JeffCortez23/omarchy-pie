# Omarchy Pie (`io.github.jeffcortez23.omarchy-pie`)

A fast, context-aware **radial pie menu** for [Omarchy](https://omarchy.org/) (Hyprland + Quickshell), powered by a native **Rust** backend.

Select any file in **Nautilus** (or copy a file/screenshot to your clipboard), press **`Super + Z`**, and a radial action wheel opens right at your cursor with tools tailored to that exact file format.

---

## ✨ Features

* **100% File-Type Aware**: Dynamically builds slices and nested radial sub-menus (`▸`) based on the exact file extension:
  * **Images (`.png`, `.jpg`, `.webp`, `.avif`, `.gif`, `.bmp`)**:
    * **Convert ▸**: WebP, PNG, JPG, AVIF, `.ICO` Favicon (64×64), Grayscale.
    * **Scale ▸**: `50%`, `75%`, `25%`, `1080p Full HD`, `720p HD`, `512×512 Square Avatar` (Lanczos3).
    * **Compress Weight**: Reduces file size in MB/KB while keeping the exact original dimensions.
    * **Transform ▸**: Rotate 90° CW/CCW, Rotate 180°, Flip Horizontal, **Extract 5 Dominant HEX Colors**, **Strip EXIF / GPS Metadata**.
    * **OCR Text**: Extracts text from images straight to your Wayland clipboard via Tesseract.
  * **Instant Sharing (`Compartir ▸` & 1-Click LocalSend)**:
    * **LocalSend**: Sends the selected file or folder to nearby devices via `localsend --headless send`.
    * **Wi-Fi QR Server**: Spins up an ephemeral local HTTP server in Rust and displays a scannable SVG QR code right in the center of the radial menu.
    * **Cloud Upload (`0x0.st`)**: Uploads file and copies the temporary link + displays a QR code.
    * **Base64 / Data URI**: Encodes images or files as `data:<mime>;base64,...` directly to clipboard.
  * **Filename Normalizer (`Normalizar ▸`)**:
    * `kebab-case`, `snake_case`, `camelCase`, `YYYY-MM-DD_name` ISO date prefix, **Clean Clutter** (removes `(1)`, `Copy of`, etc.), and `lowercase`.
  * **Video & Audio (`.mp4`, `.mkv`, `.mov`, `.webm`, `.mp3`, `.wav`, `.flac`, `.opus`)**:
    * Video to optimized palette **GIF**, extract **MP3 192k**, compress video (`<720p H.264`), mute audio track, capture PNG frame, convert to MP4/WebM/Opus/FLAC, and EBU R128 loudness normalization (`loudnorm`).
  * **Data, Docs, Code & 3D (`.csv`, `.json`, `.md`, `.pdf`, `.sh`, `.py`, `.rs`, `.blend`, `.zip`, `.tar.zst`)**:
    * **CSV ➔ JSON** converter, row/column/word counter, JSON pretty-formatter & minifier, **Markdown ➔ Styled HTML**, **PDF ➔ PNG pages**, `chmod +x`, run script in terminal, open in **Neovim** or **Blender**, extract/list archives, and **SHA-256** checksums.

---

## 🚀 Installation

### Via Omarchy Plugin CLI

```bash
omarchy plugin add https://github.com/JeffCortez23/omarchy-pie.git --enable
```

When enabled, `Service.qml` automatically symlinks `~/.local/bin/omarchy-pie` and installs the **Nautilus** right-click context script (`Scripts ➔ Omarchy Pie`).

### Recommended Hyprland Keybinding (`~/.config/hypr/bindings.lua`)

Add this to `~/.config/hypr/bindings.lua` so pressing **`Super + Z`** automatically captures the selected file in Nautilus (or uses the clipboard/latest screenshot) and opens the radial wheel at your cursor:

```lua
local paths = require("default.hypr.paths")

o.bind("SUPER + Z", "Omarchy Pie (Radial File Menu)", function()
  local win = hl.get_active_window()
  local class = win and (win.class or ""):lower() or ""
  if class:find("nautilus") or class:find("thunar") or class:find("dolphin") or class:find("nemo") or class:find("pcmanfm") then
    hl.dispatch(hl.dsp.send_key_state({ mods = "CTRL", key = "C", state = "down" }))
    hl.timer(function()
      hl.dispatch(hl.dsp.send_key_state({ mods = "CTRL", key = "C", state = "up" }))
      hl.timer(function()
        hl.exec_cmd(paths.home .. "/.local/bin/omarchy-pie")
      end, { timeout = 45, type = "oneshot" })
    end, { timeout = 35, type = "oneshot" })
  else
    hl.exec_cmd(paths.home .. "/.local/bin/omarchy-pie")
  end
end)
```

---

## 🛠️ Building from Source

```bash
cargo build --release
mkdir -p bin ~/.local/bin
cp target/release/omarchy-pie bin/omarchy-pie
cp target/release/omarchy-pie ~/.local/bin/omarchy-pie
```

## 📄 License

MIT © [Jeff Cortez](https://github.com/JeffCortez23)

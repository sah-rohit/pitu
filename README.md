# pitu 📷⚡

> **pitu**: A fast, scriptable CLI Image Workbench & Content-Aware Processing Engine for batch manipulation, AI entropy cropping, quality enhancement, file size compression, version-controlled image syncing, and CI/CD pipelines.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](#cross-platform)

---

```text
  ██████╗ ██╗████████╗██╗   ██╗
  ██╔══██╗██║╚══██╔══╝██║   ██║
  ██████╔╝██║   ██║   ██║   ██║
  ██╔═══╝ ██║   ██║   ██║   ██║
  ██║     ██║   ██║   ╚██████╔╝
  ╚═╝     ╚═╝   ╚═╝    ╚═════╝ 
  PITU WORKBENCH v0.1.0 • Scriptable CLI Image Engine
```

---

## 💡 Why `pitu`?

Opening heavy GUI editors (Photoshop, GIMP, Figma) for batch cropping, format conversion, or watermarking breaks developer flow and automation pipelines. `pitu` provides a lightning-fast terminal tool engineered in Rust for automated, content-aware image processing workflows.

### 🌟 Key Highlights

- 🧠 **Smart Entropy Cropping**: Content-aware cropping using Sobel edge detection & 2D local Shannon entropy ($O(1)$ integral image SAT) to keep the visually interesting focal point—not just the geometric center.
- ✨ **Quality Enhancement Engine**: Unsharp mask edge sharpening, contrast normalization, and color pop.
- 📉 **Target File Size Compression**: Binary search quality optimizer (`--max-size 500KB` / `--max-size 2MB`) fitting images under exact byte limits for web uploads.
- 🔄 **Continuous Interactive Edit Session**: Chain multiple actions together (`Crop` ➔ `Enhance` ➔ `Watermark`) with full **Undo (Ctrl+Z)** and **Redo (Ctrl+Y)** stack management.
- 📜 **Built-in Snapshot Versioning Sync (`pitu sync`)**: Version-controlled image snapshot commits with timestamps and operation history.
- 🔬 **Next-Gen Universal Codec Engine**: Scans magic bytes, decodes Base64 Data URIs, extracts polyglot streams, and auto-repairs corrupted file headers.
- 💾 **Save Strategy & Location Wizard**: Paste custom paths, safe `Save as Copy` (`filename_copy.png`), or overwrite with 1-click GUI File Manager launcher (`xdg-open` / `open` / `explorer`).
- ⚡ **Ultra-Fast Parallel Processing**: Multi-threaded execution across hundreds of images powered by Rayon.
- 📋 **Preset Configuration Engine (`pitu.toml`)**: Reusable pipeline presets (`web-hero`, `social-avatar`, `thumbnail-webp`).

---

## 🛠️ Installation

### Quick Launcher Setup (Recommended)
```bash
./install.sh
```
This builds the release binary and installs `pitu` to `~/.local/bin/pitu`. You can then launch `pitu` from any terminal tab!

### Build from Source
```bash
git clone https://github.com/sah-rohit/pitu.git
cd pitu
cargo build --release
```

---

## 🚀 Quick Usage Examples

### 1. Launch Interactive Drag & Drop Workbench
```bash
pitu
```

### 2. Smart Crop & Convert
```bash
pitu process "photos/*.jpg" -s 16:9 -t webp -q 85 -o dist/
```

### 3. Target File Size Compression (< 500 KB)
```bash
pitu process hero.png --max-size 500KB -o web/
```

### 4. Side-by-Side Terminal Visual Diff
```bash
pitu diff sample.jpg
```

### 5. Snapshot Commit Sync
```bash
pitu sync photo.jpg -m "Retouched header contrast"
pitu history photo.jpg
```

---

## 📄 License
Distributed under the MIT License. See [LICENSE](LICENSE) for details.

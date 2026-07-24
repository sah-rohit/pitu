# pitu 📷⚡

> **pitu**: A fast, scriptable CLI Image Workbench & Content-Aware Processing Engine for batch manipulation, AI entropy cropping, quality enhancement, file size compression, version-controlled image syncing, and CI/CD pipelines.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](#cross-platform)
[![Troubleshooting](https://img.shields.io/badge/docs-Troubleshooting%20Guide-green.svg)](TROUBLESHOOTING.md)

---

```text
  ██████╗ ██╗████████╗██╗   ██╗
  ██╔══██╗██║╚══██╔══╝██║   ██║
  ██████╔╝██║   ██║   ██║   ██║
  ██╔═══╝ ██║   ██║   ██║   ██║
  ██║     ██║   ██║   ╚██████╔╝
  ╚═╝     ╚═╝   ╚═╝    ╚═════╝ 
  PITU WORKBENCH v1.0.0 • Holy Grail CLI Image Engine
```

---

## 💡 Why `pitu`?

Opening heavy GUI editors (Photoshop, GIMP, Figma) for batch cropping, format conversion, or watermarking breaks developer flow and automation pipelines. `pitu` provides a lightning-fast terminal tool engineered in Rust for automated, content-aware image processing workflows.

### 🌟 Key Highlights

- 🖥️ **Dual-Mode CLI + Native Desktop GUI Workbench**: Launch headless CLI batch processing, terminal TUI, or modern Native Desktop GUI app window (`pitu gui` / `pitu --gui`).
- 📈 **Photoshop-inspired Spline Curves Editor**: Real-time smooth tone curves using **Cubic Hermite Spline Interpolation** with shadows, midtones, and highlights adjustments.
- 📐 **Interactive Dashed Bounding Box Crop**: Real-time dashed crop box overlay with interactive keyboard tuning and corner selection.
- 🏷️ **Rotated & Styled Watermarks**: Visual text watermarking with full control over rotation angle, opacity, scaling, and offset parameters.
- 🖱️ **Terminal Mouse click Capture**: Translate raw console click locations directly to exact high-resolution image coordinates for spot healing and selective radial masking.
- 🛡️ **Fully Non-Destructive In-Memory Workbench**: Edits are stored safely inside independent per-image session histories under `.pitu/`. Your original image files are never touched or modified on disk until you choose to export them.
- 🧠 **Smart Entropy Cropping**: Content-aware cropping using Sobel edge detection & 2D local Shannon entropy SAT to keep the visually interesting focal point—not just the geometric center.
- 📉 **Target File Size Compression**: Binary search quality optimizer (`--max-size 500KB` / `--max-size 2MB`) fitting images under exact byte limits for web uploads.
- 🔄 **Continuous Interactive Edit Session**: Chain multiple actions together with full **Undo** and **Redo** stack management.
- 📜 **Built-in Snapshot Versioning Sync (`pitu sync`)**: Version-controlled image snapshot commits with timestamps and operation history.
- 🔬 **Next-Gen Universal Codec Engine**: Scans magic bytes, decodes Base64 Data URIs, extracts polyglot streams, and auto-repairs corrupted file headers.
- 💾 **Save Strategy & Location Wizard**: Paste custom paths, safe `Save as Copy` (`filename_copy.png`), or overwrite.
- ⚡ **Ultra-Fast Parallel Processing**: Multi-threaded execution across hundreds of images powered by Rayon.

---

## 📋 Prerequisites

Before installing `pitu`, ensure your system meets the OS-specific requirements below:

| Requirement | Windows | macOS | Linux |
| :--- | :--- | :--- | :--- |
| **Rust Toolchain** | [Rustup 1.75+](https://rustup.rs) | [Rustup 1.75+](https://rustup.rs) | [Rustup 1.75+](https://rustup.rs) |
| **Build Compiler** | MSVC C++ Build Tools | Xcode Command Line Tools | `build-essential` / `base-devel` |
| **Shell / Terminal** | PowerShell 5.1+ / Windows Terminal | Terminal.app / iTerm2 | Bash / Zsh |
| **GUI Folder Launcher** | Windows Explorer (Native) | Finder (`open` native) | `xdg-utils` (`xdg-open`) |

---

## 🛠️ Cross-Platform Installation & Setup

### 🍎 macOS Setup (Terminal / iTerm2)
```bash
# 1. Install Xcode Command Line Tools (If not already installed)
xcode-select --install

# 2. Run automated macOS installer
chmod +x install.sh && ./install.sh
```

### 🪟 Windows Setup (PowerShell)
Open PowerShell and run:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; .\install.ps1
```
*Note: Our installer is ASCII-safe to prevent any PowerShell character encoding conflicts on standard Windows machines.*

### 🐧 Linux Setup (Ubuntu / Fedora / Arch)
```bash
chmod +x install.sh && ./install.sh
```

### 📦 Install via Cargo (All OS)
```bash
cargo install --path .
```

---

## 🚀 Quick Usage Examples

### 1. Launch Interactive Terminal Workbench
```bash
pitu interactive
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

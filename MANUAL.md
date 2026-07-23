# 📖 `pitu` Complete User Manual & Reference Guide

Welcome to the comprehensive user manual for **`pitu` (CLI Image Workbench)**. This document provides step-by-step guidance on how to run `pitu`, paste image locations, process batch jobs, use smart entropy cropping, and set up a custom one-line command to launch `pitu` from anywhere in your terminal.

---

## 📑 Table of Contents
1. [Quick Start & Easy Launch Setup](#1-quick-start--easy-launch-setup)
2. [Interactive Mode & Click-to-Select Presets](#2-interactive-mode--click-to-select-presets)
3. [Pasting Image Locations & Drag-and-Drop](#3-pasting-image-locations--drag-and-drop)
4. [Smart Entropy Focal-Point Cropping](#4-smart-entropy-focal-point-cropping)
5. [Batch Processing & Wildcard Globs](#5-batch-processing--wildcard-globs)
6. [Supported Formats & Transcoding](#6-supported-formats--transcoding)
7. [Watermarks, Filters & Adjustments](#7-watermarks-filters--adjustments)
8. [CI/CD Integration & JSON Output](#8-cicd-integration--json-output)
9. [Command Cheatsheet](#9-command-cheatsheet)

---

## 1. Quick Start & Easy Launch Setup

### Installing the Easy `pitu` Launcher Command
You can make `pitu` executable from **any terminal window or folder** without typing full file paths:

Run inside the project folder:
```bash
./target/release/pitu install-launcher
```

Or run the shell installer script:
```bash
./install.sh
```

Now, simply typing `pitu` in any terminal window launches the interactive workbench!

---

## 2. Interactive Mode & Click-to-Select Presets

If you don't want to memorize command flags, simply run `pitu` with **no parameters**:

```bash
pitu
```

An interactive menu appears where you can navigate using **arrow keys** and press **Enter**:

- 🖼️ **Quick 16:9 Smart Entropy Crop**: Preserves the visually interesting focal point while cropping to widescreen.
- 🌐 **Quick Convert to WebP**: Converts any image to lightweight WebP format.
- 📱 **Quick 1:1 Square Social Thumbnail**: Creates square avatar/post crops centered on the focal area.
- 🏷️ **Quick Text Watermark**: Overlays text watermarks onto photos.
- 🎨 **Quick Grayscale & Contrast Boost**: Applies instant visual filter styling.
- ⚙️ **Full Custom Pipeline**: Chain multiple operations interactively.
- 📖 **View User Manual & Documentation**: Displays full built-in manual directly in the terminal.
- ℹ️ **About pitu & Supported Features**: Displays program architecture and format specs.

---

## 3. Pasting Image Locations & Drag-and-Drop

`pitu` automatically cleans and parses pasted image locations.

### How to paste or add images:
1. **Drag and Drop**: Drag an image file or folder directly from your File Manager into the terminal prompt.
2. **Copy/Paste Path**: Copy a file location (e.g. `/home/user/Downloads/photo.jpg`) and paste it into the prompt (`Ctrl+Shift+V` or right-click paste).
3. **Browser File URLs**: Pasted URLs like `file:///home/user/Pictures/image.png` are automatically sanitized.
4. **Quoted Paths & Escaped Spaces**: Paths containing spaces (e.g. `"./My Photos/cat.jpg"` or `./My\ Photos/cat.jpg`) are parsed seamlessly.

---

## 4. Smart Entropy Focal-Point Cropping

Unlike basic crop tools that crop the geometric center (which often cuts off faces or objects near borders), `pitu` evaluates visual interest:

### Smart Crop Math & Engine
1. **Edge Magnitude**: Computes 3x3 Sobel gradient vector magnitude $G(x,y)$.
2. **Shannon Entropy**: Computes 2D local texture entropy $H(x,y)$ over pixel neighborhoods.
3. **Summed-Area Table (Integral Image)**: Calculates 2D integral sums to evaluate candidate crop boxes in $O(1)$ constant time.
4. **Focal Point Selection**: Chooses the window containing the maximum sum of edge detail and entropy.

### Commands:
```bash
pitu smart-crop photo.jpg -o cropped.jpg --ratio 16:9
pitu smart-crop photo.jpg -o square.jpg --ratio 1:1
pitu smart-crop photo.jpg -w 800 -H 600
```

---

## 5. Batch Processing & Wildcard Globs

Process hundreds of images simultaneously using parallel multithreading (`rayon`):

```bash
# Process all JPG files in a folder
pitu process "photos/*.jpg" -o ./dist --smart-crop 16:9

# Process nested folders matching multiple extensions
pitu process "assets/**/*.{jpg,png}" -o ./dist --format webp --watermark-text "© 2026 Pitu"
```

---

## 6. Supported Formats & Transcoding

`pitu` supports 7 common image formats with zero external codec dependencies:

- **PNG** (`.png`)
- **JPEG** (`.jpg`, `.jpeg`)
- **WebP** (`.webp`)
- **GIF** (`.gif`)
- **BMP** (`.bmp`)
- **TIFF** (`.tiff`, `.tif`)
- **ICO** (`.ico`)

### Format Conversion Command:
```bash
pitu convert input.png -t webp
pitu convert "images/*.jpg" -t png
```

---

## 7. Watermarks, Filters & Adjustments

### Watermarking
Overlay text or logo images aligned to 9 anchor points (`TopLeft`, `TopCenter`, `TopRight`, `CenterLeft`, `Center`, `CenterRight`, `BottomLeft`, `BottomCenter`, `BottomRight`):

```bash
# Text watermark
pitu watermark photo.jpg --text "© Pitu Workbench" --anchor bottom-right --opacity 0.8

# Image watermark overlay
pitu watermark photo.jpg --image logo.png --anchor top-left --scale 0.15
```

### Visual Filters
```bash
pitu filter photo.jpg --grayscale
pitu filter photo.jpg --sepia
pitu filter photo.jpg --blur 2.5
pitu filter photo.jpg --brightness 10 --contrast 20.0
```

---

## 8. CI/CD Integration & JSON Output

For build scripts and automated pipelines:
- `--silent`: Suppress progress bars and messages.
- `--json`: Output machine-readable JSON status.
- `--dry-run`: Simulate processing without writing output files.

```bash
pitu process "dist/*.png" --format webp --silent --json
```

---

## 9. Command Cheatsheet

| Command | Example |
| :--- | :--- |
| **Interactive Wizard** | `pitu` or `pitu interactive` |
| **Info / About** | `pitu info` |
| **User Manual** | `pitu manual` |
| **Install Global Launcher**| `pitu install-launcher` |
| **Smart Crop 16:9** | `pitu smart-crop img.jpg -o out.jpg --ratio 16:9` |
| **Convert Format** | `pitu convert img.png -t webp` |
| **Resize** | `pitu resize img.jpg -w 800` |
| **Watermark Text** | `pitu watermark img.jpg -t "Text" -a bottom-right` |
| **Shell Completions** | `pitu completions zsh` |

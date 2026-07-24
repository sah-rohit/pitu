use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use image::DynamicImage;

use crate::operations::enhance::enhance_image;
use crate::operations::filter::{apply_filters, FilterOptions};
use crate::operations::smart_crop::{smart_crop, parse_aspect_ratio, SmartCropOptions};
use crate::operations::watermark::{apply_watermark, WatermarkOptions};
use crate::cli::AnchorPosition;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SessionOperation {
    Grayscale,
    Sepia,
    Invert,
    Brightness(i32),
    Contrast(f32),
    Blur(f32),
    Sharpen(f32),
    Warmth(f32),
    Vignette(f32),
    Structure(f32),
    HdrScape,
    GlamourGlow,
    HazeRemoval,
    Frame(u32),
    SmartCrop(String), // aspect ratio e.g. "16:9"
    Watermark(String), // text
    Enhance(f32),      // strength
    Exposure(f32),
    Saturation(f32),
    Shadows(f32),
    Highlights(f32),
    Noir,
    Vintage,
    Grunge,
    LensBlur(f32),
    Healing(u32, u32, u32), // cx, cy, r
    Selective(u32, u32, u32, f32, f32), // cx, cy, r, exp, sat
    DoubleExposure(String, String), // second_path, mode
}

impl SessionOperation {
    pub fn description(&self) -> String {
        match self {
            Self::Grayscale => "Grayscale".to_string(),
            Self::Sepia => "Sepia Tone".to_string(),
            Self::Invert => "Invert Colors".to_string(),
            Self::Brightness(b) => format!("Brightness ({})", b),
            Self::Contrast(c) => format!("Contrast ({:.1})", c),
            Self::Blur(s) => format!("Gaussian Blur ({:.1})", s),
            Self::Sharpen(s) => format!("Sharpen ({:.1})", s),
            Self::Warmth(w) => format!("Color Warmth ({:.2})", w),
            Self::Vignette(v) => format!("Vignette ({:.2})", v),
            Self::Structure(s) => format!("Structure/Clarity ({:.2})", s),
            Self::HdrScape => "HDR Scape".to_string(),
            Self::GlamourGlow => "Glamour Glow".to_string(),
            Self::HazeRemoval => "Haze Removal".to_string(),
            Self::Frame(w) => format!("Border Frame ({}px)", w),
            Self::SmartCrop(ratio) => format!("Smart Entropy Crop ({})", ratio),
            Self::Watermark(txt) => format!("Watermark '{}'", txt),
            Self::Enhance(strg) => format!("Details Enhance ({:.2})", strg),
            Self::Exposure(exp) => format!("Exposure ({:.2})", exp),
            Self::Saturation(sat) => format!("Saturation ({:.2})", sat),
            Self::Shadows(sh) => format!("Shadows ({:.2})", sh),
            Self::Highlights(hi) => format!("Highlights ({:.2})", hi),
            Self::Noir => "Noir filter".to_string(),
            Self::Vintage => "Vintage filter".to_string(),
            Self::Grunge => "Grunge filter".to_string(),
            Self::LensBlur(lb) => format!("Lens Blur ({:.2})", lb),
            Self::Healing(cx, cy, r) => format!("Spot Healing at ({}, {}) r:{}", cx, cy, r),
            Self::Selective(cx, cy, r, exp, sat) => format!("Selective Mask at ({}, {}) r:{} exp:{:.1} sat:{:.1}", cx, cy, r, exp, sat),
            Self::DoubleExposure(path, mode) => format!("Double Exposure '{}' ({})", path, mode),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotEntry {
    pub hash: String,
    pub timestamp_sec: u64,
    pub message: String,
    pub original_file: String,
    pub snapshot_file: String,
    pub operation: Option<SessionOperation>,
    pub enabled: bool,
}

pub fn get_history_dir(base_path: &Path) -> PathBuf {
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let file_stem = base_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "image".into());
    parent.join(".pitu").join(format!("{}_history", file_stem))
}

pub fn initialize_history_if_needed(image_path: &Path) -> anyhow::Result<()> {
    let history_dir = get_history_dir(image_path);
    let log_file = history_dir.join("history.json");

    if !log_file.exists() {
        fs::create_dir_all(&history_dir)?;
        
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let base_target = history_dir.join("base_original.png");
        if image_path.exists() {
            let img = image::open(image_path)?;
            img.save(&base_target)?;
        }

        let base_entry = SnapshotEntry {
            hash: "base".to_string(),
            timestamp_sec: now_sec,
            message: "Initial Base Image".to_string(),
            original_file: image_path.display().to_string(),
            snapshot_file: base_target.display().to_string(),
            operation: None,
            enabled: true,
        };

        let history = vec![base_entry];
        let updated_json = serde_json::to_string_pretty(&history)?;
        fs::write(log_file, updated_json)?;
    }

    Ok(())
}

pub fn apply_session_operation(img: &DynamicImage, op: &SessionOperation) -> anyhow::Result<DynamicImage> {
    match op {
        SessionOperation::Grayscale => {
            let fopts = FilterOptions { grayscale: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Sepia => {
            let fopts = FilterOptions { sepia: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Invert => {
            let fopts = FilterOptions { invert: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Brightness(b) => {
            let fopts = FilterOptions { brightness: Some(*b), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Contrast(c) => {
            let fopts = FilterOptions { contrast: Some(*c), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Blur(s) => {
            let fopts = FilterOptions { blur: Some(*s), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Sharpen(s) => {
            let fopts = FilterOptions { sharpen: Some(*s), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Warmth(w) => {
            let fopts = FilterOptions { warmth: Some(*w), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Vignette(v) => {
            let fopts = FilterOptions { vignette: Some(*v), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Structure(s) => {
            let fopts = FilterOptions { structure: Some(*s), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::HdrScape => {
            let fopts = FilterOptions { hdr_scape: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::GlamourGlow => {
            let fopts = FilterOptions { glamour_glow: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::HazeRemoval => {
            let fopts = FilterOptions { haze_removal: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Frame(w) => {
            let fopts = FilterOptions { frame_width: Some(*w), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::SmartCrop(ratio) => {
            let aspect = parse_aspect_ratio(ratio);
            Ok(smart_crop(img, &SmartCropOptions {
                target_width: None,
                target_height: None,
                aspect_ratio: aspect,
                entropy_weight: 0.5,
            }))
        }
        SessionOperation::Watermark(txt) => {
            apply_watermark(img, &WatermarkOptions {
                text: Some(txt.clone()),
                image_path: None,
                anchor: AnchorPosition::BottomRight,
                opacity: 0.8,
                scale: 0.2,
            })
        }
        SessionOperation::Enhance(strg) => {
            Ok(enhance_image(img, *strg))
        }
        SessionOperation::Exposure(exp) => {
            let fopts = FilterOptions { exposure: Some(*exp), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Saturation(sat) => {
            let fopts = FilterOptions { saturation: Some(*sat), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Shadows(sh) => {
            let fopts = FilterOptions { shadows: Some(*sh), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Highlights(hi) => {
            let fopts = FilterOptions { highlights: Some(*hi), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Noir => {
            let fopts = FilterOptions { noir: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Vintage => {
            let fopts = FilterOptions { vintage: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Grunge => {
            let fopts = FilterOptions { grunge: true, ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::LensBlur(lb) => {
            let fopts = FilterOptions { lens_blur: Some(*lb), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Healing(cx, cy, r) => {
            let fopts = FilterOptions { healing: Some((*cx, *cy, *r)), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::Selective(cx, cy, r, exp, sat) => {
            let fopts = FilterOptions { selective: Some((*cx, *cy, *r, *exp, *sat)), ..Default::default() };
            Ok(apply_filters(img, &fopts))
        }
        SessionOperation::DoubleExposure(path, mode) => {
            let fopts = FilterOptions {
                double_exposure_path: Some(PathBuf::from(path)),
                double_exposure_mode: Some(mode.clone()),
                ..Default::default()
            };
            Ok(apply_filters(img, &fopts))
        }
    }
}

pub fn create_snapshot(image_path: &Path, message: &str, operation: Option<SessionOperation>) -> anyhow::Result<SnapshotEntry> {
    initialize_history_if_needed(image_path)?;

    let history_dir = get_history_dir(image_path);
    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let file_stem = image_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "image".into());

    let hash = format!("{:x}", now_sec).chars().take(8).collect::<String>();
    let snapshot_filename = format!("{}_{}.webp", file_stem, hash);
    let snapshot_target = history_dir.join(&snapshot_filename);

    if image_path.exists() {
        let img = image::open(image_path)?;
        let _ = img.save_with_format(&snapshot_target, image::ImageFormat::WebP);
    }

    let entry = SnapshotEntry {
        hash: hash.clone(),
        timestamp_sec: now_sec,
        message: message.to_string(),
        original_file: image_path.display().to_string(),
        snapshot_file: snapshot_target.display().to_string(),
        operation,
        enabled: true,
    };

    let log_file = history_dir.join("history.json");
    let mut history: Vec<SnapshotEntry> = {
        let content = fs::read_to_string(&log_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    };

    history.push(entry.clone());
    let updated_json = serde_json::to_string_pretty(&history)?;
    fs::write(log_file, updated_json)?;

    // Optional local Git sync: if image_path is inside a git repo, stage and commit!
    let parent = image_path.parent().unwrap_or_else(|| Path::new("."));
    let is_git_repo = std::process::Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .current_dir(parent)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if is_git_repo {
        // Stage the modified image file
        let _ = std::process::Command::new("git")
            .args(&["add", image_path.to_str().unwrap_or_default()])
            .current_dir(parent)
            .status();

        // Commit with the operation message
        let commit_msg = format!("pitu: {}", message);
        let _ = std::process::Command::new("git")
            .args(&["commit", "-m", &commit_msg])
            .current_dir(parent)
            .status();
    }

    Ok(entry)
}

pub fn list_history(image_path: &Path) -> Vec<SnapshotEntry> {
    let history_dir = get_history_dir(image_path);
    let log_file = history_dir.join("history.json");
    if !log_file.exists() {
        return Vec::new();
    }
    let content = fs::read_to_string(log_file).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn rebuild_image_from_history(image_path: &Path) -> anyhow::Result<DynamicImage> {
    let history = list_history(image_path);
    if history.is_empty() {
        return Ok(image::open(image_path)?);
    }

    // First entry is "base_original.png"
    let base_entry = &history[0];
    let mut img = image::open(&base_entry.snapshot_file)?;

    for entry in history.iter().skip(1) {
        if entry.enabled {
            if let Some(ref op) = entry.operation {
                img = apply_session_operation(&img, op)?;
            }
        }
    }

    Ok(img)
}

pub fn rebase_history(image_path: &Path, indexes_to_disable: &[usize]) -> anyhow::Result<()> {
    initialize_history_if_needed(image_path)?;
    let history_dir = get_history_dir(image_path);
    let log_file = history_dir.join("history.json");

    let mut history: Vec<SnapshotEntry> = {
        let content = fs::read_to_string(&log_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    };

    for &idx in indexes_to_disable {
        if idx < history.len() {
            history[idx].enabled = false;
        }
    }

    let updated_json = serde_json::to_string_pretty(&history)?;
    fs::write(&log_file, updated_json)?;

    // Rebuild final image
    let rebuilt_img = rebuild_image_from_history(image_path)?;
    rebuilt_img.save(image_path)?;

    // Save snapshot of the rebase outcome
    let _ = create_snapshot(image_path, "Interactive Rebase Outcome", None)?;

    Ok(())
}

pub fn revert_to_commit(image_path: &Path, commit_ref: &str) -> anyhow::Result<()> {
    initialize_history_if_needed(image_path)?;
    let history_dir = get_history_dir(image_path);
    let log_file = history_dir.join("history.json");

    let history: Vec<SnapshotEntry> = {
        let content = fs::read_to_string(&log_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    };

    // Find entry by hash or by index (if numeric)
    let found_entry = if let Ok(idx) = commit_ref.parse::<usize>() {
        history.get(idx).cloned()
    } else {
        history.iter().find(|e| e.hash.starts_with(commit_ref)).cloned()
    };

    if let Some(entry) = found_entry {
        let img = image::open(&entry.snapshot_file)?;
        img.save(image_path)?;
        let _ = create_snapshot(image_path, &format!("Reverted to snapshot [{}]", entry.hash), None)?;
        Ok(())
    } else {
        anyhow::bail!("Snapshot commit reference '{}' not found.", commit_ref)
    }
}

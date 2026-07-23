use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotEntry {
    pub hash: String,
    pub timestamp_sec: u64,
    pub message: String,
    pub original_file: String,
    pub snapshot_file: String,
}

pub fn get_history_dir(base_path: &Path) -> PathBuf {
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(".pitu").join("history")
}

pub fn create_snapshot(image_path: &Path, message: &str) -> anyhow::Result<SnapshotEntry> {
    let history_dir = get_history_dir(image_path);
    fs::create_dir_all(&history_dir)?;

    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let file_stem = image_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "image".into());

    let hash = format!("{:x}", now_sec);
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
    };

    let log_file = history_dir.join("history.json");
    let mut history: Vec<SnapshotEntry> = if log_file.exists() {
        let content = fs::read_to_string(&log_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    history.push(entry.clone());
    let updated_json = serde_json::to_string_pretty(&history)?;
    fs::write(log_file, updated_json)?;

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

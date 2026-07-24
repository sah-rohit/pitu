use crate::cli::ImageFormatChoice;
use console::{style, Emoji};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");
static CHECK: Emoji<'_, '_> = Emoji("✔ ", "[OK] ");
static CROSS: Emoji<'_, '_> = Emoji("✖ ", "[ERR] ");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessReport {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_ms: u128,
    pub items: Vec<ProcessItemResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessItemResult {
    pub input_path: String,
    pub output_path: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u128,
}

pub fn print_banner(silent: bool) {
    if silent {
        return;
    }
    println!(
        "{} {}",
        style("pitu").bold().cyan(),
        style("v0.1.0 - CLI Image Workbench").dim()
    );
}

pub fn print_success(msg: &str, silent: bool) {
    if !silent {
        println!("{} {}", CHECK, style(msg).green().bold());
    }
}

pub fn print_info(msg: &str, silent: bool) {
    if !silent {
        println!("{} {}", SPARKLES, style(msg).cyan());
    }
}

pub fn print_error(msg: &str, silent: bool) {
    if !silent {
        eprintln!("{} {}", CROSS, style(msg).red().bold());
    }
}

/// Resolve destination output path for a given input file path
pub fn resolve_output_path(
    input_path: &Path,
    output_target: Option<&Path>,
    target_format: Option<ImageFormatChoice>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> PathBuf {
    let parent_dir = input_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "image".into());

    let original_ext = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "png".to_string());

    let ext = target_format
        .map(|f| f.extension())
        .unwrap_or(&original_ext);

    let new_filename = format!(
        "{}{}{}.{}",
        prefix.unwrap_or(""),
        stem,
        suffix.unwrap_or(""),
        ext
    );

    let final_path = if let Some(target) = output_target {
        let target_str = target.to_string_lossy();
        if target.is_dir() || target_str.ends_with('/') || target_str.ends_with('\\') || target.extension().is_none() {
            target.join(new_filename)
        } else {
            target.to_path_buf()
        }
    } else {
        parent_dir.join(new_filename)
    };

    if let Some(parent) = final_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    final_path
}

/// Robust path sanitizer for pasted locations, dragged files, file URLs, and escaped spaces
pub fn sanitize_input_path(raw: &str) -> String {
    let mut cleaned = raw.trim();

    // Strip leading file:// or file:///
    if cleaned.starts_with("file:///") {
        cleaned = &cleaned[7..];
    } else if cleaned.starts_with("file://") {
        cleaned = &cleaned[7..];
    }

    // Strip quotes
    cleaned = cleaned.trim_matches('\'').trim_matches('"');

    // Replace escaped spaces '\ ' with regular space ' '
    let unescaped = cleaned.replace("\\ ", " ");

    unescaped.trim().to_string()
}

/// Print formatted JSON report to stdout
pub fn print_json_report(report: &ProcessReport) {
    if let Ok(json) = serde_json::to_string_pretty(report) {
        println!("{}", json);
    }
}

/// Calculate Shannon Information Entropy of an image (luminance details measure)
pub fn calculate_entropy(img: &image::DynamicImage) -> f32 {
    let gray = img.to_luma8();
    let total = gray.len() as f32;
    let mut hist = [0u32; 256];
    for p in gray.pixels() {
        hist[p[0] as usize] += 1;
    }
    let mut entropy = 0.0f32;
    for &count in &hist {
        if count > 0 {
            let p = count as f32 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Calculate structural similarity index (SSIM) between two images (fast 128x128 comparison)
pub fn calculate_ssim(img1: &image::DynamicImage, img2: &image::DynamicImage) -> f32 {
    let sub1 = img1.thumbnail(128, 128).to_luma8();
    let sub2 = img2.thumbnail(128, 128).to_luma8();
    let n = sub1.len() as f64;
    if n == 0.0 {
        return 1.0;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for (p1, p2) in sub1.pixels().zip(sub2.pixels()) {
        sum_x += p1[0] as f64;
        sum_y += p2[0] as f64;
    }
    let mu_x = sum_x / n;
    let mu_y = sum_y / n;

    let mut var_x = 0.0;
    let mut var_y = 0.0;
    let mut cov_xy = 0.0;
    for (p1, p2) in sub1.pixels().zip(sub2.pixels()) {
        let x = p1[0] as f64;
        let y = p2[0] as f64;
        var_x += (x - mu_x) * (x - mu_x);
        var_y += (y - mu_y) * (y - mu_y);
        cov_xy += (x - mu_x) * (y - mu_y);
    }
    let sigma_x_sq = var_x / (n - 1.0);
    let sigma_y_sq = var_y / (n - 1.0);
    let sigma_xy = cov_xy / (n - 1.0);

    let c1 = (0.01 * 255.0) * (0.01 * 255.0);
    let c2 = (0.03 * 255.0) * (0.03 * 255.0);

    let numerator = (2.0 * mu_x * mu_y + c1) * (2.0 * sigma_xy + c2);
    let denominator = (mu_x * mu_x + mu_y * mu_y + c1) * (sigma_x_sq + sigma_y_sq + c2);

    let ssim = numerator / denominator;
    ssim.clamp(0.0, 1.0) as f32
}

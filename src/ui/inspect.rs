use crate::ui::ascii_preview::render_ascii_thumbnail;
use console::style;
use image::GenericImageView;
use std::fs;
use std::path::Path;

pub fn render_image_inspector(file_path: &Path) -> anyhow::Result<()> {
    let metadata = fs::metadata(file_path)?;
    let file_size_kb = metadata.len() as f64 / 1024.0;

    let img = image::open(file_path)?;
    let (w, h) = img.dimensions();
    let color_type = format!("{:?}", img.color());
    let aspect_ratio = w as f64 / h as f64;

    let format_str = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".into());

    println!("\n  {}", style("╭── 🔍 IMAGE INSPECTOR DASHBOARD ──────────────────────────────╮").cyan().bold());
    println!("  │  File Path     : {:<42} │", style(file_path.display()).yellow());
    println!("  │  Dimensions    : {:<42} │", style(format!("{} x {} px", w, h)).green().bold());
    println!("  │  Aspect Ratio  : {:<42} │", style(format!("{:.2}:1", aspect_ratio)).cyan());
    println!("  │  File Size     : {:<42} │", style(format!("{:.2} KB", file_size_kb)).magenta());
    println!("  │  Format        : {:<42} │", style(format_str).bold());
    println!("  │  Color Space   : {:<42} │", style(color_type).dim());
    println!("  {}\n", style("╰──────────────────────────────────────────────────────────────╯").cyan().bold());

    print!("{}", render_ascii_thumbnail(&img, 50));

    Ok(())
}

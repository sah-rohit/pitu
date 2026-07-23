use crate::operations::Pipeline;
use console::{style, Term};
use image::{DynamicImage, GenericImageView};
use std::path::Path;

/// Render true-color ANSI ASCII thumbnail preview of an image in terminal
pub fn render_ascii_thumbnail(img: &DynamicImage, mut target_width: u32) -> String {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return "Empty image".to_string();
    }

    let term_width = Term::stdout().size().1 as u32;
    if term_width > 10 {
        target_width = target_width.min(term_width.saturating_sub(10)).max(20);
    }

    let aspect = (h as f32 / w as f32) * 0.5;
    let target_height = ((target_width as f32 * aspect).round() as u32).max(1);

    let resized = img.resize_exact(target_width, target_height, image::imageops::FilterType::Triangle);
    let rgba = resized.to_rgba8();

    let mut out = String::new();
    out.push_str(&format!("  ┌── Terminal Preview ({}x{}) {}\n", w, h, "─".repeat(target_width.saturating_sub(24) as usize)));

    for y in 0..target_height {
        out.push_str("  │");
        for x in 0..target_width {
            let p = rgba.get_pixel(x, y);
            if p[3] < 32 {
                out.push(' ');
            } else {
                let ansi_color = format!("\x1b[38;2;{};{};{}m█\x1b[0m", p[0], p[1], p[2]);
                out.push_str(&ansi_color);
            }
        }
        out.push_str("│\n");
    }

    out.push_str(&format!("  └──{}──┘\n", "─".repeat(target_width as usize)));
    out
}

/// Render Smart Crop Entropy Heatmap preview in terminal ASCII
pub fn render_entropy_heatmap_preview(img: &DynamicImage, mut target_width: u32) -> String {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return "Empty image".to_string();
    }

    let term_width = Term::stdout().size().1 as u32;
    if term_width > 10 {
        target_width = target_width.min(term_width.saturating_sub(10)).max(20);
    }

    let aspect = (h as f32 / w as f32) * 0.5;
    let target_height = ((target_width as f32 * aspect).round() as u32).max(1);

    let resized = img.resize_exact(target_width, target_height, image::imageops::FilterType::Triangle);
    let gray = resized.to_luma8();

    let mut energy = vec![0.0f32; (target_width * target_height) as usize];
    let mut max_e = 1e-5f32;

    for y in 1..target_height.saturating_sub(1) {
        for x in 1..target_width.saturating_sub(1) {
            let p00 = gray.get_pixel(x - 1, y - 1)[0] as f32;
            let p02 = gray.get_pixel(x + 1, y - 1)[0] as f32;
            let p10 = gray.get_pixel(x - 1, y)[0] as f32;
            let p12 = gray.get_pixel(x + 1, y)[0] as f32;
            let p20 = gray.get_pixel(x - 1, y + 1)[0] as f32;
            let p22 = gray.get_pixel(x + 1, y + 1)[0] as f32;

            let gx = (-1.0 * p00) + (1.0 * p02) + (-2.0 * p10) + (2.0 * p12) + (-1.0 * p20) + (1.0 * p22);
            let gy = (-1.0 * p00) + (1.0 * p20) + (-2.0 * p10) + (2.0 * p12) + (1.0 * p22);
            let mag = (gx * gx + gy * gy).sqrt();

            let idx = (y * target_width + x) as usize;
            energy[idx] = mag;
            if mag > max_e {
                max_e = mag;
            }
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "  ┌── {} {}\n",
        style("Smart Entropy Energy Map").yellow().bold(),
        "─".repeat(target_width.saturating_sub(26) as usize)
    ));

    for y in 0..target_height {
        out.push_str("  │");
        for x in 0..target_width {
            let idx = (y * target_width + x) as usize;
            let norm = energy[idx] / max_e;

            let (r, g, b, ch) = if norm > 0.75 {
                (255, 40, 80, '█')
            } else if norm > 0.50 {
                (240, 220, 0, '▓')
            } else if norm > 0.25 {
                (50, 220, 50, '▒')
            } else if norm > 0.10 {
                (0, 180, 220, '░')
            } else {
                (20, 30, 100, ' ')
            };

            let ansi_pixel = format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, ch);
            out.push_str(&ansi_pixel);
        }
        out.push_str("│\n");
    }

    out.push_str(&format!("  └──{}──┘\n", "─".repeat(target_width as usize)));
    out.push_str("  Legend: \x1b[38;2;255;40;80m█ High Focal Interest\x1b[0m  \x1b[38;2;240;220;0m▓ Medium\x1b[0m  \x1b[38;2;50;220;50m▒ Low\x1b[0m  \x1b[38;2;20;30;100m  Background\x1b[0m\n\n");

    out
}

/// Render Side-by-Side Visual Diff comparison (Original vs Processed)
pub fn render_side_by_side_diff(orig: &DynamicImage, proc_img: &DynamicImage) -> String {
    let (orig_w, orig_h) = orig.dimensions();
    let (proc_w, proc_h) = proc_img.dimensions();

    let col_w = 32u32;
    let h1 = ((col_w as f32 * (orig_h as f32 / orig_w as f32) * 0.5).round() as u32).max(1);
    let h2 = ((col_w as f32 * (proc_h as f32 / proc_w as f32) * 0.5).round() as u32).max(1);
    let target_h = h1.max(h2).min(20);

    let res_orig = orig.resize_exact(col_w, target_h, image::imageops::FilterType::Triangle).to_rgba8();
    let res_proc = proc_img.resize_exact(col_w, target_h, image::imageops::FilterType::Triangle).to_rgba8();

    let mut out = String::new();
    out.push_str(&format!(
        "\n  {}",
        style("╭── 👁️ SIDE-BY-SIDE VISUAL DIFF COMPARISON ─────────────────────────────────╮").cyan().bold()
    ));
    out.push_str(&format!(
        "\n  │ {:^32}     {:^32} │\n",
        style(format!("ORIGINAL ({}x{})", orig_w, orig_h)).yellow().bold(),
        style(format!("PROCESSED ({}x{})", proc_w, proc_h)).green().bold()
    ));

    out.push_str(&format!(
        "  │ ┌{}┐   ┌{}┐ │\n",
        "─".repeat(col_w as usize),
        "─".repeat(col_w as usize)
    ));

    for y in 0..target_h {
        out.push_str("  │ │");
        for x in 0..col_w {
            let p = res_orig.get_pixel(x, y);
            out.push_str(&format!("\x1b[38;2;{};{};{}m█\x1b[0m", p[0], p[1], p[2]));
        }
        out.push_str("│   │");
        for x in 0..col_w {
            let p = res_proc.get_pixel(x, y);
            out.push_str(&format!("\x1b[38;2;{};{};{}m█\x1b[0m", p[0], p[1], p[2]));
        }
        out.push_str("│ │\n");
    }

    out.push_str(&format!(
        "  │ └{}┘   └{}┘ │\n",
        "─".repeat(col_w as usize),
        "─".repeat(col_w as usize)
    ));
    out.push_str(&format!("  {}\n\n", style("╰───────────────────────────────────────────────────────────────────────────╯").cyan().bold()));

    out
}

pub fn render_diff_cmd(file_path: &Path, pipeline: &Pipeline) -> anyhow::Result<()> {
    let orig = image::open(file_path)?;
    let processed = pipeline.execute(&orig)?;

    print!("{}", render_side_by_side_diff(&orig, &processed));
    Ok(())
}

pub fn render_preview_cmd(file_path: &Path, heatmap: bool) -> anyhow::Result<()> {
    let img = image::open(file_path)?;
    println!("\n  Previewing file: {}\n", style(file_path.display()).cyan().bold());

    if heatmap {
        print!("{}", render_entropy_heatmap_preview(&img, 50));
    } else {
        print!("{}", render_ascii_thumbnail(&img, 50));
        print!("{}", render_entropy_heatmap_preview(&img, 50));
    }

    Ok(())
}

use crate::cli::ImageFormatChoice;
use crate::utils::sanitize_input_path;
use console::style;
use inquire::{Select, Text};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum NamingStrategy {
    OverwriteOriginal,
    SaveAsCopy,
    CustomName(String),
}

pub struct SaveOptions {
    pub destination_dir: PathBuf,
    pub naming_strategy: NamingStrategy,
    pub format: Option<ImageFormatChoice>,
}

/// Prompt user for output location, naming strategy, and format choices
pub fn prompt_save_options(default_input_path: &Path) -> SaveOptions {
    println!("\n  {}", style("💾 OUTPUT SAVE LOCATION & STRATEGY WIZARD").cyan().bold());
    println!("  ───────────────────────────────────────────────────────────");

    let default_parent = default_input_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy();

    let dest_input = Text::new("Save Location / Folder:")
        .with_default(&default_parent)
        .with_placeholder("Paste folder path or press Enter for same directory")
        .prompt()
        .unwrap_or_else(|_| default_parent.to_string());

    let sanitized_dest = sanitize_input_path(&dest_input);
    let destination_dir = PathBuf::from(if sanitized_dest.is_empty() {
        ".".to_string()
    } else {
        sanitized_dest
    });

    let strategy_choices = vec![
        "📋  Save as Copy (e.g. filename_copy.png - safe, no overwrite)",
        "⚠️  Overwrite Original File (replace original in-place)",
        "✏️  Custom New Filename",
    ];

    let strategy_selection = Select::new("Choose File Naming Strategy:", strategy_choices)
        .prompt()
        .unwrap_or("📋  Save as Copy (e.g. filename_copy.png - safe, no overwrite)");

    let naming_strategy = if strategy_selection.contains("Overwrite Original") {
        NamingStrategy::OverwriteOriginal
    } else if strategy_selection.contains("Custom New Filename") {
        let custom_name = Text::new("Enter Custom Filename (without extension):")
            .with_placeholder("e.g. my_processed_photo")
            .prompt()
            .unwrap_or_else(|_| "processed_image".to_string());
        NamingStrategy::CustomName(custom_name)
    } else {
        NamingStrategy::SaveAsCopy
    };

    let fmt_choices = vec![
        "Preserve / Auto-Detect Format",
        "🌐 WebP (Web optimized - small size, high quality)",
        "🖼️ JPEG (High compatibility photo format)",
        "🎨 PNG (Lossless with transparent alpha support)",
        "🎞️ GIF (Animated or indexed color)",
        "📄 BMP (Uncompressed bitmap)",
        "📐 TIFF (High precision publishing format)",
        "🖼️ ICO (Icon file format)",
    ];

    let fmt_sel = Select::new("Select Target Extension / Format:", fmt_choices)
        .prompt()
        .unwrap_or("Preserve / Auto-Detect Format");

    let format = if fmt_sel.contains("WebP") {
        Some(ImageFormatChoice::Webp)
    } else if fmt_sel.contains("JPEG") {
        Some(ImageFormatChoice::Jpeg)
    } else if fmt_sel.contains("PNG") {
        Some(ImageFormatChoice::Png)
    } else if fmt_sel.contains("GIF") {
        Some(ImageFormatChoice::Gif)
    } else if fmt_sel.contains("BMP") {
        Some(ImageFormatChoice::Bmp)
    } else if fmt_sel.contains("TIFF") {
        Some(ImageFormatChoice::Tiff)
    } else if fmt_sel.contains("ICO") {
        Some(ImageFormatChoice::Ico)
    } else {
        None
    };

    SaveOptions {
        destination_dir,
        naming_strategy,
        format,
    }
}

/// Compute exact target output file path based on save options
pub fn compute_target_path(
    input_file: &Path,
    save_opts: &SaveOptions,
) -> PathBuf {
    let stem = input_file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());

    let orig_ext = input_file
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "png".to_string());

    let ext = save_opts
        .format
        .map(|f| f.extension().to_string())
        .unwrap_or(orig_ext);

    let new_filename = match &save_opts.naming_strategy {
        NamingStrategy::OverwriteOriginal => format!("{}.{}", stem, ext),
        NamingStrategy::CustomName(custom) => format!("{}.{}", custom, ext),
        NamingStrategy::SaveAsCopy => {
            let base_copy = format!("{}_copy.{}", stem, ext);
            let target_candidate = save_opts.destination_dir.join(&base_copy);

            if target_candidate.exists() {
                let mut idx = 1;
                loop {
                    let cand = save_opts.destination_dir.join(format!("{}_copy_{}.{}", stem, idx, ext));
                    if !cand.exists() {
                        break cand.file_name().unwrap().to_string_lossy().to_string();
                    }
                    idx += 1;
                }
            } else {
                base_copy
            }
        }
    };

    let final_path = save_opts.destination_dir.join(new_filename);
    if let Some(parent) = final_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    final_path
}

/// Open target file or output folder in OS Desktop File Manager (GUI)
pub fn open_file_manager(target_path: &Path) -> anyhow::Result<()> {
    let folder = if target_path.is_dir() {
        target_path
    } else {
        target_path.parent().unwrap_or_else(|| Path::new("."))
    };

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(folder).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(folder).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(folder).spawn()?;
    }

    println!(
        "  📂 Launched desktop file manager for location: {}",
        style(folder.display()).cyan().bold()
    );

    Ok(())
}

/// Prompt post-save navigation choices (Open file manager, view terminal preview, dashboard)
pub fn post_save_action_prompt(saved_file_path: &Path) {
    println!(
        "\n  ✔ {} Saved output file to: {}",
        style("SUCCESS!").green().bold(),
        style(saved_file_path.display()).cyan().bold()
    );

    let choices = vec![
        "📁  Open Folder in Desktop File Manager (GUI)",
        "👁️   View Truecolor ASCII Terminal Preview",
        "↩️   Return to Main Dashboard",
    ];

    let sel = Select::new("Choose next action:", choices)
        .prompt()
        .unwrap_or("↩️   Return to Main Dashboard");

    if sel.contains("Open Folder in Desktop") {
        let _ = open_file_manager(saved_file_path);
    } else if sel.contains("View Truecolor ASCII") {
        let _ = crate::ui::ascii_preview::render_preview_cmd(saved_file_path, false);
    }
}

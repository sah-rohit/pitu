use console::style;
use std::fs;
use std::env;
use std::path::PathBuf;

pub fn show_info_screen() {
    println!("\n{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").cyan());
    println!("  {} - {}", style("pitu").cyan().bold(), style("CLI Image Workbench v0.1.0").bold());
    println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").cyan());
    println!("  {}", style("Fast, scriptable terminal tool for common image operations.").dim());
    println!("  {}", style("Designed for desktop ease-of-use & CI/CD pipeline automation.").dim());

    println!("\n{}", style("✨ KEY FEATURES").bold().yellow());
    println!("  • {}", style("Smart Entropy Crop:").bold());
    println!("    Content-aware crop using Sobel edge detection & local Shannon entropy");
    println!("    evaluated via 2D Integral Images (Summed Area Tables) in O(1) time.");
    println!("  • {}", style("Multi-threaded Batching:").bold());
    println!("    Parallel CPU processing powered by Rayon for hundreds of images.");
    println!("  • {}", style("Zero-Typing Interactive Mode:").bold());
    println!("    Drag & drop paths, paste image locations, and select presets with arrow keys.");
    println!("  • {}", style("Watermarking & Filters:").bold());
    println!("    Image & text overlays with 9-point anchor alignment, sepia, blur, grayscale.");
    println!("  • {}", style("CI/CD Automation:").bold());
    println!("    Structured JSON reports (--json), quiet mode (--silent), and exit code status.");

    println!("\n{}", style("🖼️ SUPPORTED IMAGE FORMATS").bold().yellow());
    println!("  • PNG (.png)   • JPEG (.jpg, .jpeg)   • WebP (.webp)");
    println!("  • GIF (.gif)   • BMP (.bmp)           • TIFF (.tiff, .tif)");
    println!("  • ICO (.ico)");

    println!("\n{}", style("🛠️ SYSTEM LAUNCHER").bold().yellow());
    println!("  Run '{}' to make 'pitu' available in any terminal window.", style("pitu install-launcher").green());
    println!("{}\n", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").cyan());
}

pub fn show_manual_screen() {
    println!("\n{}", style("📖 PITU USER MANUAL & CHEATSHEET").bold().cyan());
    println!("────────────────────────────────────────────────────────────────────");
    println!("{}", style("1. LAUNCHING PITU").bold().yellow());
    println!("   • Type 'pitu' anywhere to open the Interactive Wizard.");
    println!("   • Type 'pitu install-launcher' to install the 'pitu' command globally.");

    println!("\n{}", style("2. PASTING FILE LOCATIONS & DRAG-AND-DROP").bold().yellow());
    println!("   • You can paste file paths directly into the terminal.");
    println!("   • Supported input types:");
    println!("     - Dragged files from file manager (e.g. '/home/user/Pictures/photo.jpg')");
    println!("     - Browser file URLs (e.g. 'file:///home/user/photo.jpg')");
    println!("     - Quoted paths (e.g. \"./My Photos/cat.png\")");
    println!("     - Wildcard globs (e.g. \"./photos/*.{{jpg,png}}\")");
    println!("     - Entire directories (e.g. \"./my_images_folder\")");

    println!("\n{}", style("3. SMART ENTROPY CROPPING").bold().yellow());
    println!("   • Preserves visual focal point instead of cropping center.");
    println!("   • Examples:");
    println!("     pitu smart-crop input.png -o output.png --ratio 16:9");
    println!("     pitu smart-crop photo.jpg --ratio 1:1");
    println!("     pitu process \"*.jpg\" --smart-crop 4:3");

    println!("\n{}", style("4. BATCH PROCESSING & FORMAT CONVERSION").bold().yellow());
    println!("   • Convert format:");
    println!("     pitu convert \"*.png\" -t webp");
    println!("   • Resize:");
    println!("     pitu resize \"*.jpg\" -w 800 -H 600");
    println!("   • Watermark:");
    println!("     pitu watermark \"*.png\" --text \"© 2026 Pitu\" --anchor bottom-right");
    println!("   • Full Pipeline:");
    println!("     pitu process \"photos/*.jpg\" -o ./dist --smart-crop 16:9 --watermark-text \"Pitu\" --format webp");

    println!("\n{}", style("5. CI/CD INTEGRATION").bold().yellow());
    println!("   • Generate machine-readable JSON status:");
    println!("     pitu process \"dist/*.png\" --format webp --silent --json");
    println!("────────────────────────────────────────────────────────────────────\n");
}

pub fn install_global_launcher() -> anyhow::Result<()> {
    let current_exe = env::current_exe()?;
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;

    let local_bin = PathBuf::from(&home_dir).join(".local").join("bin");
    let cargo_bin = PathBuf::from(&home_dir).join(".cargo").join("bin");

    let target_dir = if local_bin.exists() || fs::create_dir_all(&local_bin).is_ok() {
        local_bin
    } else {
        cargo_bin
    };

    let target_path = target_dir.join("pitu");

    let _ = fs::remove_file(&target_path);
    fs::copy(&current_exe, &target_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }

    println!(
        "\n{} Installed launcher binary to: {}",
        console::Emoji("✔ ", "[OK] "),
        style(target_path.display()).green().bold()
    );
    println!(
        "  You can now type '{}' from any terminal tab!\n",
        style("pitu").cyan().bold()
    );

    Ok(())
}

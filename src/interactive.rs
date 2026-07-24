use crate::cli::ProcessArgs;
use crate::manual::{show_info_screen, show_manual_screen};
use crate::operations::compress::{compress_to_max_size, parse_size_bytes};
use crate::operations::enhance::enhance_image;
use crate::operations::universal_reader::load_universal_image;
use crate::operations::Pipeline;
use crate::ui::ascii_preview::{render_preview_cmd, render_side_by_side_diff};
use crate::ui::banner::{print_footer_hints, print_header_banner, print_welcome_dashboard};
use crate::ui::exporter::{compute_target_path, post_save_action_prompt, prompt_save_options};
use crate::ui::inspect::render_image_inspector;
use image::{DynamicImage, GenericImageView};
use crate::utils::{print_error, print_info, print_success};
use crate::versioning::{
    create_snapshot, list_history, rebuild_image_from_history, revert_to_commit, SessionOperation,
};
use console::style;
use inquire::{Confirm, Select, Text};
use std::fs;
use std::path::Path;

fn prompt_back_to_dashboard() -> bool {
    println!();
    let choices = vec!["↩️  Return to Main Dashboard", "🚪  Exit Pitu"];
    match Select::new("Choose next action:", choices).prompt() {
        Ok(c) => c.contains("Return to Main Dashboard"),
        Err(_) => false,
    }
}

pub fn run_interactive_wizard() -> anyhow::Result<()> {
    loop {
        print_header_banner();
        print_welcome_dashboard();

        let action_choices = vec![
            "─── 🔄 Continuous Workbench & Session ───",
            "🎨  Continuous Edit Session (Chain Operations with Auto-Save & Rebase)",
            "─── 🧠 Smart AI & Quality Enhancements ───",
            "✨  Enhance Quality, Sharpness & Color Pop",
            "📉  Compress File to Target Max Size (e.g. < 500KB, 2MB)",
            "🖼️  Quick 16:9 Smart Entropy Crop (Focal-point preserved)",
            "📱  Quick 1:1 Square Social Thumbnail",
            "─── ⚙️ Core Operations ───",
            "📋  Execute Preset Workflow (web-hero, social-avatar, etc.)",
            "🌐  Quick Convert to WebP (Web optimized)",
            "🏷️  Quick Text Watermark Overlay",
            "🎨  Quick Grayscale & Contrast Boost",
            "⚙️  Full Custom Pipeline & Operations",
            "─── 👁️ Visual Previews & Inspection ───",
            "👁️  Side-by-Side Visual Diff (Original vs Processed)",
            "🔍  Inspect Image Metadata & Color Specs",
            "🗺️  View Terminal Entropy Heatmap Preview",
            "─── 📜 Versioning & Sync ───",
            "🛠️  Interactive Rebase (Selectively toggle/remove intermediate effects)",
            "↩️  Revert Image to Any Snapshot Commit",
            "📜  Create Snapshot Commit Sync & View History Timeline",
            "─── 📖 Help & Config ───",
            "📝  Generate Starter pitu.toml Configuration",
            "📖  View User Manual & Documentation",
            "ℹ️  About pitu Telemetry & Coders Info",
            "─── 🚪 System ───",
            "🚪  Exit Pitu",
        ];

        print_footer_hints();

        let choice_result = Select::new("Select Action / Operation:", action_choices).prompt();
        let mut choice = match choice_result {
            Ok(c) => c,
            Err(_) => {
                println!("\n  Goodbye!");
                return Ok(());
            }
        };

        // Handle section dividers if user highlights them
        while choice.starts_with("───") {
            println!("  Please select an actionable operation from the menu below.\n");
            let retry = Select::new("Select Action / Operation:", vec![
                "🎨  Continuous Edit Session (Chain Operations with Auto-Save & Rebase)",
                "✨  Enhance Quality, Sharpness & Color Pop",
                "📉  Compress File to Target Max Size (e.g. < 500KB, 2MB)",
                "🖼️  Quick 16:9 Smart Entropy Crop (Focal-point preserved)",
                "📱  Quick 1:1 Square Social Thumbnail",
                "📋  Execute Preset Workflow (web-hero, social-avatar, etc.)",
                "🌐  Quick Convert to WebP (Web optimized)",
                "🏷️  Quick Text Watermark Overlay",
                "🎨  Quick Grayscale & Contrast Boost",
                "⚙️  Full Custom Pipeline & Operations",
                "👁️  Side-by-Side Visual Diff (Original vs Processed)",
                "🔍  Inspect Image Metadata & Color Specs",
                "🗺️  View Terminal Entropy Heatmap Preview",
                "🛠️  Interactive Rebase (Selectively toggle/remove intermediate effects)",
                "↩️  Revert Image to Any Snapshot Commit",
                "📜  Create Snapshot Commit Sync & View History Timeline",
                "📝  Generate Starter pitu.toml Configuration",
                "📖  View User Manual & Documentation",
                "ℹ️  About pitu Telemetry & Coders Info",
                "🚪  Exit Pitu",
            ]).prompt();

            choice = match retry {
                Ok(c) => c,
                Err(_) => return Ok(()),
            };
        }

        if choice.contains("Exit Pitu") {
            println!("\n  Goodbye!");
            return Ok(());
        }

        if choice.contains("User Manual") {
            show_manual_screen();
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("About pitu") {
            show_info_screen();
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Generate Starter pitu.toml") {
            let _ = crate::config::create_default_config_file();
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        println!("\n  📌 Paste image location, drag & drop files/folders, or enter a glob pattern.");
        let input_path_raw = match Text::new("Target Image / Folder / Location:")
            .with_placeholder("e.g. paste location or drag file here")
            .prompt()
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let input_pattern = crate::utils::sanitize_input_path(&input_path_raw);

        if input_pattern.is_empty() {
            print_error("No input location provided.", false);
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        let input_path_obj = Path::new(&input_pattern);

        if choice.contains("Create Snapshot Commit Sync") {
            let history = list_history(input_path_obj);
            println!("\n  📜 VERSION COMMIT HISTORY TIMELINE for: {}", style(&input_pattern).cyan().bold());
            println!("  ───────────────────────────────────────────────────────────");
            if history.is_empty() {
                println!("  No previous snapshot commits found.");
            } else {
                for entry in &history {
                    println!(
                        "  ▫ [{}] {} - {}",
                        style(&entry.hash).cyan().bold(),
                        style(&entry.message).green(),
                        style(format!("{}s ago", entry.timestamp_sec)).dim()
                    );
                }
            }
            println!();

            let do_sync = Confirm::new("Create a new Snapshot Commit Sync for this file?")
                .with_default(true)
                .prompt()
                .unwrap_or(false);

            if do_sync {
                let msg = Text::new("Commit Message:").with_default("Updated image snapshot").prompt().unwrap_or_else(|_| "Snapshot".into());
                if let Ok(entry) = create_snapshot(input_path_obj, &msg, None) {
                    print_success(&format!("Snapshot created! Hash: [{}]", entry.hash), false);
                }
            }

            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Interactive Rebase") {
            if let Err(e) = run_interactive_rebase(input_path_obj) {
                print_error(&format!("Rebase error: {}", e), false);
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Revert Image to Any Snapshot Commit") {
            if let Err(e) = run_interactive_revert(input_path_obj) {
                print_error(&format!("Revert error: {}", e), false);
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Continuous Edit Session") {
            if let Ok(read_res) = load_universal_image(input_path_obj) {
                // Initialize history
                let _ = crate::versioning::initialize_history_if_needed(input_path_obj);
                let history = list_history(input_path_obj);
                
                let mut current_img = read_res.image;
                let original_img = if !history.is_empty() {
                    image::open(&history[0].snapshot_file).unwrap_or_else(|_| current_img.clone())
                } else {
                    current_img.clone()
                };

                loop {
                    let history = list_history(input_path_obj);

                    // Print beautiful Original vs Processed side-by-side terminal dashboard!
                    print!("{}", render_side_by_side_diff(&original_img, &current_img));

                    println!("  📜 History Operations Log:\n");
                    for (i, entry) in history.iter().enumerate().skip(1) {
                        let status = if entry.enabled {
                            style("● active").green()
                        } else {
                            style("○ disabled").red()
                        };
                        let desc = entry.operation.as_ref().map_or("Custom Edit".to_string(), |op| op.description());
                        println!("    {}. [{}] {} - {}", i, style(&entry.hash).cyan(), status, desc);
                    }
                    println!();

                    let mut session_choices = vec![
                        "💡 Exposure (Brighten/Darken)",
                        "🎨 Saturation (Color Intensity)",
                        "🌡️ Warmth (Cool / Golden temperature)",
                        "📐 Details / Structure Clarity",
                        "🎭 Vignette Halo Border",
                        "🌅 HDR Scape Dynamic Range Boost",
                        "✨ Glamour Soft Portrait Glow",
                        "🌫️ Haze Removal De-haze",
                        "🌑 Shadows Recovery",
                        "☀️ Highlights Recovery",
                        "🎬 Noir (High-contrast B&W)",
                        "🎞️ Vintage Classic Film Tint",
                        "🎸 Grunge Scratchy Grain",
                        "👁️ Lens Blur / Bokeh Focus",
                        "🩹 Spot Healing / Blemish Removal",
                        "🎯 Selective Adjustment Circle",
                        "🎭 Double Exposure Blending",
                        "🧠 16:9 Smart Entropy Crop",
                        "📱 1:1 Square Smart Crop",
                        "🏷️ Text Watermark Overlay",
                        "🖼️ Add Border Frame",
                    ];

                    // Persistent Undo / Redo checks
                    let has_undo = history.iter().skip(1).any(|e| e.enabled);
                    let has_redo = history.iter().skip(1).any(|e| !e.enabled);

                    if has_undo {
                        session_choices.push("↩️  Undo Last Active Edit");
                    }
                    if has_redo {
                        session_choices.push("🔁  Redo Last Disabled Edit");
                    }
                    session_choices.push("🛠️  Interactive Rebase (Selective Layer Edit)");
                    session_choices.push("💾  Export / Save As Copy");
                    session_choices.push("🚪  Finish & Exit Session");

                    let sess_sel = match Select::new("Choose Adjustment / Operation:", session_choices).prompt() {
                        Ok(s) => s,
                        Err(_) => break,
                    };

                    if sess_sel.contains("Undo Last Active Edit") {
                        // Toggle last enabled to disabled
                        let mut history = list_history(input_path_obj);
                        if let Some(idx) = history.iter().rposition(|e| e.enabled && e.operation.is_some()) {
                            history[idx].enabled = false;
                            let log_file = crate::versioning::get_history_dir(input_path_obj).join("history.json");
                            let _ = fs::write(&log_file, serde_json::to_string_pretty(&history)?);
                            if let Ok(rebuilt) = rebuild_image_from_history(input_path_obj) {
                                current_img = rebuilt;
                                let _ = current_img.save(input_path_obj);
                                print_success("Undid last adjustment persistently!", false);
                            }
                        }
                    } else if sess_sel.contains("Redo Last Disabled Edit") {
                        // Toggle first disabled to enabled
                        let mut history = list_history(input_path_obj);
                        if let Some(idx) = history.iter().position(|e| !e.enabled && e.operation.is_some()) {
                            history[idx].enabled = true;
                            let log_file = crate::versioning::get_history_dir(input_path_obj).join("history.json");
                            let _ = fs::write(&log_file, serde_json::to_string_pretty(&history)?);
                            if let Ok(rebuilt) = rebuild_image_from_history(input_path_obj) {
                                current_img = rebuilt;
                                let _ = current_img.save(input_path_obj);
                                print_success("Redid adjustment persistently!", false);
                            }
                        }
                    } else if sess_sel.contains("Interactive Rebase") {
                        let _ = run_interactive_rebase(input_path_obj);
                        if let Ok(rebuilt) = rebuild_image_from_history(input_path_obj) {
                            current_img = rebuilt;
                        }
                    } else if sess_sel.contains("Finish") {
                        break;
                    } else if sess_sel.contains("Export") {
                        let save_options = prompt_save_options(input_path_obj);
                        let target_path = compute_target_path(input_path_obj, &save_options);
                        let fmt = save_options.format.unwrap_or(crate::cli::ImageFormatChoice::Webp);
                        if let Ok(bytes) = crate::operations::convert::convert_format_to_bytes(&current_img, fmt, 85) {
                            if std::fs::write(&target_path, bytes).is_ok() {
                                post_save_action_prompt(&target_path);
                            }
                        }
                    } else {
                        // Parse operation
                        let op = if sess_sel.contains("Exposure") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Exposure(v), "Exposure offset", 0.0, 0.05, -3.0, 3.0)
                        } else if sess_sel.contains("Saturation") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Saturation(v), "Saturation multiplier", 1.0, 0.05, 0.0, 3.0)
                        } else if sess_sel.contains("Warmth") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Warmth(v), "Color Warmth", 0.0, 0.05, -1.0, 1.0)
                        } else if sess_sel.contains("Structure") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Structure(v), "Structure / Micro-contrast", 0.0, 0.05, 0.0, 3.0)
                        } else if sess_sel.contains("Vignette") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Vignette(v), "Vignette strength", 0.0, 0.05, 0.0, 1.5)
                        } else if sess_sel.contains("HDR Scape") {
                            Some(SessionOperation::HdrScape)
                        } else if sess_sel.contains("Glamour") {
                            Some(SessionOperation::GlamourGlow)
                        } else if sess_sel.contains("Haze Removal") {
                            Some(SessionOperation::HazeRemoval)
                        } else if sess_sel.contains("Shadows") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Shadows(v), "Shadows level", 0.0, 0.05, -1.0, 1.0)
                        } else if sess_sel.contains("Highlights") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Highlights(v), "Highlights level", 0.0, 0.05, -1.0, 1.0)
                        } else if sess_sel.contains("Noir") {
                            Some(SessionOperation::Noir)
                        } else if sess_sel.contains("Vintage") {
                            Some(SessionOperation::Vintage)
                        } else if sess_sel.contains("Grunge") {
                            Some(SessionOperation::Grunge)
                        } else if sess_sel.contains("Lens Blur") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::LensBlur(v), "Lens Blur radius", 0.0, 0.05, 0.0, 5.0)
                        } else if sess_sel.contains("Spot Healing") {
                            let (w, h) = current_img.dimensions();
                            tune_healing_tui(&original_img, &current_img, w / 2, h / 2, 20)
                        } else if sess_sel.contains("Selective Adjustment") {
                            let (w, h) = current_img.dimensions();
                            tune_selective_tui(&original_img, &current_img, w / 2, h / 2, 50, 0.0, 1.0)
                        } else if sess_sel.contains("Double Exposure") {
                            let path_str = Text::new("Path to secondary image file to blend:").prompt().unwrap_or_default();
                            if std::path::Path::new(&path_str).exists() {
                                let mode = Select::new("Choose blend mode:", vec!["multiply", "screen", "overlay", "add"]).prompt().unwrap_or_else(|_| "overlay".into());
                                Some(SessionOperation::DoubleExposure(path_str, mode.to_string()))
                            } else {
                                print_error("File does not exist. Double exposure cancelled.", false);
                                None
                            }
                        } else if sess_sel.contains("16:9 Smart Entropy Crop") {
                            Some(SessionOperation::SmartCrop("16:9".to_string()))
                        } else if sess_sel.contains("1:1 Square Smart Crop") {
                            Some(SessionOperation::SmartCrop("1:1".to_string()))
                        } else if sess_sel.contains("Text Watermark") {
                            let t = Text::new("Watermark text:").with_default("© Pitu").prompt().unwrap_or_else(|_| "© Pitu".into());
                            Some(SessionOperation::Watermark(t))
                        } else if sess_sel.contains("Border Frame") {
                            tune_parameter_tui(&original_img, &current_img, &|v| SessionOperation::Frame(v as u32), "Border Frame width", 15.0, 1.0, 0.0, 100.0)
                        } else {
                            None
                        };

                        if let Some(operation) = op {
                            // Apply operation
                            match crate::versioning::apply_session_operation(&current_img, &operation) {
                                Ok(new_img) => {
                                    current_img = new_img;
                                    // Auto-save: overwrite original file immediately!
                                    let _ = current_img.save(input_path_obj);
                                    // Auto-commit: create persistent history snapshot automatically!
                                    let desc = operation.description();
                                    let _ = create_snapshot(input_path_obj, &desc, Some(operation));
                                    print_success(&format!("Successfully applied and auto-saved: {}", desc), false);
                                }
                                Err(e) => {
                                    print_error(&format!("Error applying operation: {}", e), false);
                                }
                            }
                        }
                    }
                }
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Enhance Quality") {
            if let Ok(read_res) = load_universal_image(input_path_obj) {
                let enhanced = enhance_image(&read_res.image, 1.2);
                let save_options = prompt_save_options(input_path_obj);
                let target_path = compute_target_path(input_path_obj, &save_options);
                let fmt = save_options.format.unwrap_or(crate::cli::ImageFormatChoice::Webp);
                if let Ok(bytes) = crate::operations::convert::convert_format_to_bytes(&enhanced, fmt, 85) {
                    if std::fs::write(&target_path, bytes).is_ok() {
                        print_success("Quality Enhancement applied!", false);
                        post_save_action_prompt(&target_path);
                    }
                }
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Compress File to Target Max Size") {
            let spec_str = Text::new("Enter Target File Size (e.g. 500KB, 2MB, 1.5MB):")
                .with_default("500KB")
                .prompt()
                .unwrap_or_else(|_| "500KB".into());

            if let Some(target_bytes) = parse_size_bytes(&spec_str) {
                if let Ok(read_res) = load_universal_image(input_path_obj) {
                    let save_options = prompt_save_options(input_path_obj);
                    let target_path = compute_target_path(input_path_obj, &save_options);
                    let fmt = save_options.format.unwrap_or(crate::cli::ImageFormatChoice::Webp);

                    if let Ok((compressed_bytes, final_q)) = compress_to_max_size(&read_res.image, target_bytes, fmt) {
                        if std::fs::write(&target_path, &compressed_bytes).is_ok() {
                            print_success(
                                &format!(
                                    "Compressed file to {} bytes (Quality: {}%) fitting under target limit!",
                                    compressed_bytes.len(), final_q
                                ),
                                false,
                            );
                            post_save_action_prompt(&target_path);
                        }
                    }
                }
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Inspect Image Metadata") {
            if let Err(e) = render_image_inspector(input_path_obj) {
                print_error(&format!("Error inspecting image '{}': {}", input_pattern, e), false);
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("View Terminal Entropy Heatmap") {
            if let Err(e) = render_preview_cmd(input_path_obj, true) {
                print_error(&format!("Error rendering heatmap for '{}': {}", input_pattern, e), false);
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Side-by-Side Visual Diff") {
            let sc_opts = ProcessArgs {
                smart_crop: Some("16:9".to_string()),
                watermark_text: Some("© Pitu Diff".to_string()),
                ..Default::default()
            };
            if let Ok(pipeline) = Pipeline::from_process_args(&sc_opts) {
                if let Err(e) = crate::ui::ascii_preview::render_diff_cmd(input_path_obj, &pipeline) {
                    print_error(&format!("Error generating diff for '{}': {}", input_pattern, e), false);
                }
            }
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        let mut process_args = ProcessArgs {
            input: input_pattern.clone(),
            ..Default::default()
        };

        if choice.contains("Execute Preset Workflow") {
            let config = crate::config::load_config();
            let preset_names: Vec<&str> = config.presets.keys().map(|k| k.as_str()).collect();
            let selected_preset = match Select::new("Choose Preset Workflow:", preset_names).prompt() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if let Some(preset) = config.presets.get(selected_preset) {
                process_args = preset.to_process_args(input_pattern.clone());
            }
        } else if choice.contains("Quick 16:9 Smart Entropy Crop") {
            process_args.smart_crop = Some("16:9".to_string());
        } else if choice.contains("Quick 1:1 Square Social Thumbnail") {
            process_args.smart_crop = Some("1:1".to_string());
        } else if choice.contains("Quick Text Watermark") {
            let txt = match Text::new("Watermark Text:").with_default("© Pitu Workbench").prompt() {
                Ok(t) => t,
                Err(_) => continue,
            };
            process_args.watermark_text = Some(txt);
        } else if choice.contains("Quick Grayscale & Contrast Boost") {
            process_args.grayscale = true;
            process_args.contrast = Some(15.0);
        } else if choice.contains("Full Custom Pipeline") {
            let do_sc = Confirm::new("Add Smart Crop?").with_default(false).prompt().unwrap_or(false);
            if do_sc {
                let r = Text::new("Smart crop ratio (e.g. 16:9, 1:1, 4:3):").with_default("16:9").prompt().unwrap_or_else(|_| "16:9".into());
                process_args.smart_crop = Some(r);
            }

            let do_resize = Confirm::new("Add Resize?").with_default(false).prompt().unwrap_or(false);
            if do_resize {
                let res = Text::new("Resize target (e.g. 800x600, 50%):").with_default("800x600").prompt().unwrap_or_else(|_| "800x600".into());
                process_args.resize = Some(res);
            }

            let do_wm = Confirm::new("Add Watermark?").with_default(false).prompt().unwrap_or(false);
            if do_wm {
                let t = Text::new("Watermark text:").with_default("Pitu").prompt().unwrap_or_else(|_| "Pitu".into());
                process_args.watermark_text = Some(t);
            }
        }

        // Interactive Save Location & Strategy Wizard
        let save_options = prompt_save_options(input_path_obj);

        let pipeline = match Pipeline::from_process_args(&process_args) {
            Ok(p) => p,
            Err(e) => {
                print_error(&format!("Pipeline error: {}", e), false);
                if !prompt_back_to_dashboard() {
                    return Ok(());
                }
                continue;
            }
        };

        let inputs = crate::batch::expand_input_paths(&[process_args.input]);

        if inputs.is_empty() {
            print_error("No matching image files found at specified location.", false);
            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        print_info(
            &format!("Processing {} image(s)...", inputs.len()),
            false,
        );

        let target_file_path = compute_target_path(input_path_obj, &save_options);

        let batch_opts = crate::batch::BatchOptions {
            output: Some(target_file_path.clone()),
            format: save_options.format,
            quality: 85,
            prefix: None,
            suffix: None,
            jobs: None,
            silent: false,
            json: false,
            dry_run: false,
            overwrite: true,
        };

        let report = crate::batch::execute_batch(&pipeline, &inputs, &batch_opts);

        if report.successful > 0 {
            print_success(
                &format!(
                    "Done! Processed {} file(s) in {}ms.",
                    report.successful, report.duration_ms
                ),
                false,
            );
            post_save_action_prompt(&target_file_path);
        } else {
            print_error(
                &format!(
                    "Failed to process file(s) (0 succeeded, {} failed). See error message above.",
                    report.failed
                ),
                false,
            );
        }

        if !prompt_back_to_dashboard() {
            return Ok(());
        }
    }
}

pub fn run_interactive_rebase(image_path: &Path) -> anyhow::Result<()> {
    crate::versioning::initialize_history_if_needed(image_path)?;
    let history_dir = crate::versioning::get_history_dir(image_path);

    loop {
        let history = list_history(image_path);
        if history.is_empty() {
            println!("  No history commits found.");
            return Ok(());
        }

        println!("\n  {}", style("🛠️  INTERACTIVE REBASE / OPERATION LAYER MANAGER").cyan().bold());
        println!("  ───────────────────────────────────────────────────────────");
        println!("  Original File: {}", style(image_path.display()).yellow());
        println!("  Operations timeline:\n");

        let mut choices = Vec::new();
        for (idx, entry) in history.iter().enumerate() {
            let status = if entry.enabled {
                style("● active  ").green()
            } else {
                style("○ disabled").red()
            };

            let op_desc = if let Some(ref op) = entry.operation {
                op.description()
            } else {
                entry.message.clone()
            };

            let label = format!(
                "  [{}] {} - {} ({})",
                idx,
                status,
                style(&op_desc).white().bold(),
                style(&entry.hash).cyan()
            );
            choices.push(label);
        }

        choices.push("✅  Apply changes & Exit Rebase".to_string());
        choices.push("❌  Discard & Exit".to_string());

        let sel = Select::new("Select an operation index to toggle, or Apply/Exit:", choices).prompt()?;

        if sel.contains("Apply changes") {
            print_success("Rebase applied successfully!", false);
            break;
        }
        if sel.contains("Discard") {
            break;
        }

        if let Some(start) = sel.find('[') {
            if let Some(end) = sel.find(']') {
                let idx_str = &sel[start+1..end];
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx == 0 {
                        println!("  Cannot toggle the initial base snapshot commit!");
                        continue;
                    }
                    let log_file = history_dir.join("history.json");
                    let mut history = list_history(image_path);
                    if idx < history.len() {
                        history[idx].enabled = !history[idx].enabled;
                        let updated_json = serde_json::to_string_pretty(&history)?;
                        fs::write(&log_file, updated_json)?;
                        
                        match rebuild_image_from_history(image_path) {
                            Ok(rebuilt) => {
                                rebuilt.save(image_path)?;
                                let base_img = image::open(&history[0].snapshot_file)?;
                                print!("{}", render_side_by_side_diff(&base_img, &rebuilt));
                                print_success(&format!("Toggled operation at index {}.", idx), false);
                            }
                            Err(e) => {
                                print_error(&format!("Error rebuilding image: {}", e), false);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn run_interactive_revert(image_path: &Path) -> anyhow::Result<()> {
    let history = list_history(image_path);
    if history.is_empty() {
        println!("  No version snapshot history found for this image.");
        return Ok(());
    }

    println!("\n  {}", style("↩️  REVERT IMAGE TO SNAPSHOT COMMIT").cyan().bold());
    println!("  ───────────────────────────────────────────────────────────");

    let mut choices = Vec::new();
    for (idx, entry) in history.iter().enumerate() {
        let label = format!(
            "  [{}] {} - {} (hash: {})",
            idx,
            style(&entry.message).yellow(),
            style(format!("{}s ago", entry.timestamp_sec)).dim(),
            style(&entry.hash).cyan()
        );
        choices.push(label);
    }
    choices.push("❌  Cancel".to_string());

    let sel = Select::new("Choose a snapshot commit to revert to:", choices).prompt()?;
    if sel.contains("Cancel") {
        return Ok(());
    }

    if let Some(start) = sel.find('[') {
        if let Some(end) = sel.find(']') {
            let idx_str = &sel[start+1..end];
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(entry) = history.get(idx) {
                    let confirm = Confirm::new(&format!("Revert file back to snapshot [{}]?", entry.hash))
                        .with_default(true)
                        .prompt()
                        .unwrap_or(false);

                    if confirm {
                        revert_to_commit(image_path, &entry.hash)?;
                        print_success(&format!("Image successfully reverted to snapshot [{}]!", entry.hash), false);
                    }
                }
            }
        }
    }

    Ok(())
}

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crate::utils::{calculate_entropy, calculate_ssim};

fn tune_parameter_tui(
    original_img: &DynamicImage,
    pre_slide_img: &DynamicImage,
    op_generator: &dyn Fn(f32) -> SessionOperation,
    param_name: &str,
    mut current_val: f32,
    step: f32,
    min_val: f32,
    max_val: f32,
) -> Option<SessionOperation> {
    if let Err(_) = enable_raw_mode() {
        return None;
    }

    let mut result_op = None;

    loop {
        let op = op_generator(current_val);
        let preview_img = match crate::versioning::apply_session_operation(pre_slide_img, &op) {
            Ok(img) => img,
            Err(_) => pre_slide_img.clone(),
        };

        print!("\x1b[2J\x1b[H");
        print!("{}", render_side_by_side_diff(original_img, &preview_img));

        let pct = ((current_val - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
        let pos = (pct * 20.0).round() as usize;
        let mut slider_bar = String::new();
        for i in 0..=20 {
            if i == pos {
                slider_bar.push('■');
            } else {
                slider_bar.push('─');
            }
        }

        let ssim = calculate_ssim(original_img, &preview_img);
        let ent_orig = calculate_entropy(original_img);
        let ent_proc = calculate_entropy(&preview_img);

        println!("  🎛️  Tuning Parameter: {}", style(param_name).cyan().bold());
        println!("  Value: [{:.2}]  {}", current_val, style(slider_bar).yellow());
        println!(
            "  📊 SSIM: {:.3} | Entropy: {:.2} (orig: {:.2})",
            ssim, ent_proc, ent_orig
        );
        println!("  Controls: [Left/Right] adjust, [Up/Down] fast adjust, [Enter] commit, [Esc] cancel");

        match event::read() {
            Ok(Event::Key(key_event)) => {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('-') => {
                        current_val = (current_val - step).max(min_val);
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('+') => {
                        current_val = (current_val + step).min(max_val);
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        current_val = (current_val + step * 5.0).min(max_val);
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        current_val = (current_val - step * 5.0).max(min_val);
                    }
                    KeyCode::Enter => {
                        result_op = Some(op);
                        break;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let _ = disable_raw_mode();
    result_op
}

fn tune_healing_tui(
    original_img: &DynamicImage,
    pre_slide_img: &DynamicImage,
    mut cx: u32,
    mut cy: u32,
    mut r: u32,
) -> Option<SessionOperation> {
    if let Err(_) = enable_raw_mode() {
        return None;
    }
    let mut result_op = None;
    let (w, h) = pre_slide_img.dimensions();

    loop {
        let op = SessionOperation::Healing(cx, cy, r);
        let preview_img = match crate::versioning::apply_session_operation(pre_slide_img, &op) {
            Ok(img) => img,
            Err(_) => pre_slide_img.clone(),
        };

        print!("\x1b[2J\x1b[H");
        print!("{}", render_side_by_side_diff(original_img, &preview_img));
        println!("  🩹 Spot Healing at Center: ({}, {}), Radius: {}", cx, cy, r);
        println!("  Controls: [Left/Right/Up/Down] move spot, [+/-] change radius, [Enter] commit, [Esc] cancel");

        match event::read() {
            Ok(Event::Key(key_event)) => {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('a') => {
                        cx = cx.saturating_sub(10).max(0);
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        cx = (cx + 10).min(w - 1);
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        cy = cy.saturating_sub(10).max(0);
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        cy = (cy + 10).min(h - 1);
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        r = (r + 2).min(200);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        r = r.saturating_sub(2).max(1);
                    }
                    KeyCode::Enter => {
                        result_op = Some(op);
                        break;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let _ = disable_raw_mode();
    result_op
}

fn tune_selective_tui(
    original_img: &DynamicImage,
    pre_slide_img: &DynamicImage,
    mut cx: u32,
    mut cy: u32,
    mut r: u32,
    mut exp: f32,
    mut sat: f32,
) -> Option<SessionOperation> {
    if let Err(_) = enable_raw_mode() {
        return None;
    }
    let mut result_op = None;
    let (w, h) = pre_slide_img.dimensions();
    let mut active_field = 0; // 0: Position, 1: Radius, 2: Exposure, 3: Saturation

    loop {
        let op = SessionOperation::Selective(cx, cy, r, exp, sat);
        let preview_img = match crate::versioning::apply_session_operation(pre_slide_img, &op) {
            Ok(img) => img,
            Err(_) => pre_slide_img.clone(),
        };

        print!("\x1b[2J\x1b[H");
        print!("{}", render_side_by_side_diff(original_img, &preview_img));

        println!("\n  🎯 Selective Radial Masking");
        println!("  ────────────────────────────");
        
        let fields = [
            format!("Center coordinates: ({}, {})", cx, cy),
            format!("Mask Radius: {}px", r),
            format!("Local Exposure: {:.2}", exp),
            format!("Local Saturation: {:.2}", sat),
        ];

        for (i, f) in fields.iter().enumerate() {
            if i == active_field {
                println!("  {} {}", style("▶").cyan().bold(), style(f).white().bold());
            } else {
                println!("    {}", style(f).dim());
            }
        }

        println!("\n  Controls: [Tab] switch field, [Left/Right/Up/Down] adjust active, [Enter] commit, [Esc] cancel");

        match event::read() {
            Ok(Event::Key(key_event)) => {
                match key_event.code {
                    KeyCode::Tab => {
                        active_field = (active_field + 1) % 4;
                    }
                    KeyCode::Left | KeyCode::Char('a') => {
                        match active_field {
                            0 => cx = cx.saturating_sub(15).max(0),
                            1 => r = r.saturating_sub(5).max(5),
                            2 => exp = (exp - 0.1).max(-3.0),
                            3 => sat = (sat - 0.1).max(0.0),
                            _ => {}
                        }
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        match active_field {
                            0 => cx = (cx + 15).min(w - 1),
                            1 => r = (r + 5).min(500),
                            2 => exp = (exp + 0.1).min(3.0),
                            3 => sat = (sat + 0.1).min(3.0),
                            _ => {}
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        match active_field {
                            0 => cy = cy.saturating_sub(15).max(0),
                            1 => r = (r + 15).min(500),
                            2 => exp = (exp + 0.3).min(3.0),
                            3 => sat = (sat + 0.3).min(3.0),
                            _ => {}
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        match active_field {
                            0 => cy = (cy + 15).min(h - 1),
                            1 => r = r.saturating_sub(15).max(5),
                            2 => exp = (exp - 0.3).max(-3.0),
                            3 => sat = (sat - 0.3).max(0.0),
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        result_op = Some(op);
                        break;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let _ = disable_raw_mode();
    result_op
}

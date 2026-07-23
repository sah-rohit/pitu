use crate::cli::ProcessArgs;
use crate::manual::{show_info_screen, show_manual_screen};
use crate::operations::compress::{compress_to_max_size, parse_size_bytes};
use crate::operations::enhance::enhance_image;
use crate::operations::universal_reader::load_universal_image;
use crate::operations::Pipeline;
use crate::session::EditSession;
use crate::ui::ascii_preview::{render_ascii_thumbnail, render_diff_cmd, render_preview_cmd};
use crate::ui::banner::{print_footer_hints, print_header_banner, print_welcome_dashboard};
use crate::ui::exporter::{compute_target_path, post_save_action_prompt, prompt_save_options};
use crate::ui::inspect::render_image_inspector;
use crate::utils::{print_error, print_info, print_success};
use console::style;
use inquire::{Confirm, Select, Text};
use std::path::{Path, PathBuf};

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
            "🎨  Continuous Edit Session (Chain Operations with Undo & Redo)",
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
                "🎨  Continuous Edit Session (Chain Operations with Undo & Redo)",
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
            let history = crate::versioning::list_history(input_path_obj);
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
                if let Ok(entry) = crate::versioning::create_snapshot(input_path_obj, &msg) {
                    print_success(&format!("Snapshot created! Hash: [{}]", entry.hash), false);
                }
            }

            if !prompt_back_to_dashboard() {
                return Ok(());
            }
            continue;
        }

        if choice.contains("Continuous Edit Session") {
            if let Ok(read_res) = load_universal_image(input_path_obj) {
                let mut session = EditSession::new(read_res.image);

                loop {
                    print!("{}", render_ascii_thumbnail(&session.current_image, 40));
                    println!("  📜 History Chain: {}\n", style(session.history_log.join(" ➔ ")).dim());

                    let mut session_choices = vec![
                        "✨ Add Quality Enhancement & Sharpening",
                        "🖼️ Add 16:9 Smart Entropy Crop",
                        "📱 Add 1:1 Square Crop",
                        "🎨 Add Contrast & Grayscale Boost",
                        "🏷️ Add Text Watermark",
                    ];

                    if session.can_undo() {
                        session_choices.push("↩️  Undo Last Operation (Ctrl+Z)");
                    }
                    if session.can_redo() {
                        session_choices.push("🔁  Redo Operation (Ctrl+Y)");
                    }
                    session_choices.push("💾  Save & Export Result");
                    session_choices.push("🚪  Finish Session");

                    let sess_sel = match Select::new("Choose Operation to Chain:", session_choices).prompt() {
                        Ok(s) => s,
                        Err(_) => break,
                    };

                    if sess_sel.contains("Undo") {
                        if session.undo() {
                            print_success("Undid last operation!", false);
                        }
                    } else if sess_sel.contains("Redo") {
                        if session.redo() {
                            print_success("Redid operation!", false);
                        }
                    } else if sess_sel.contains("Enhance") {
                        let enhanced = enhance_image(&session.current_image, 1.2);
                        session.apply_action(enhanced, "Quality Enhanced".to_string());
                    } else if sess_sel.contains("16:9 Smart Entropy Crop") {
                        let opts = ProcessArgs { smart_crop: Some("16:9".to_string()), ..Default::default() };
                        if let Ok(p) = Pipeline::from_process_args(&opts) {
                            if let Ok(processed) = p.execute(&session.current_image) {
                                session.apply_action(processed, "16:9 Crop".to_string());
                            }
                        }
                    } else if sess_sel.contains("1:1 Square Crop") {
                        let opts = ProcessArgs { smart_crop: Some("1:1".to_string()), ..Default::default() };
                        if let Ok(p) = Pipeline::from_process_args(&opts) {
                            if let Ok(processed) = p.execute(&session.current_image) {
                                session.apply_action(processed, "1:1 Crop".to_string());
                            }
                        }
                    } else if sess_sel.contains("Contrast & Grayscale") {
                        let opts = ProcessArgs { grayscale: true, contrast: Some(15.0), ..Default::default() };
                        if let Ok(p) = Pipeline::from_process_args(&opts) {
                            if let Ok(processed) = p.execute(&session.current_image) {
                                session.apply_action(processed, "Grayscale+Contrast".to_string());
                            }
                        }
                    } else if sess_sel.contains("Text Watermark") {
                        let txt = Text::new("Watermark Text:").with_default("© Pitu").prompt().unwrap_or_else(|_| "Pitu".into());
                        let opts = ProcessArgs { watermark_text: Some(txt), ..Default::default() };
                        if let Ok(p) = Pipeline::from_process_args(&opts) {
                            if let Ok(processed) = p.execute(&session.current_image) {
                                session.apply_action(processed, "Watermark".to_string());
                            }
                        }
                    } else if sess_sel.contains("Save & Export") {
                        let save_options = prompt_save_options(input_path_obj);
                        let target_path = compute_target_path(input_path_obj, &save_options);
                        let fmt = save_options.format.unwrap_or(crate::cli::ImageFormatChoice::Webp);
                        if let Ok(bytes) = crate::operations::convert::convert_format_to_bytes(&session.current_image, fmt, 85) {
                            if std::fs::write(&target_path, bytes).is_ok() {
                                post_save_action_prompt(&target_path);
                            }
                        }
                    } else {
                        break;
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

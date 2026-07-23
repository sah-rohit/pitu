mod batch;
mod cli;
mod config;
mod gui;
mod interactive;
mod manual;
mod operations;
mod session;
mod ui;
mod utils;
mod versioning;

use batch::{execute_batch, expand_input_paths, BatchOptions};
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, ProcessArgs};
use config::{create_default_config_file, load_config};
use gui::run_gui;
use manual::{install_global_launcher, show_info_screen, show_manual_screen};
use operations::Pipeline;
use std::io;
use std::path::Path;
use std::process::exit;
use ui::ascii_preview::{render_diff_cmd, render_preview_cmd};
use ui::inspect::render_image_inspector;
use utils::{print_banner, print_error, print_json_report, print_success};
use versioning::{create_snapshot, list_history};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check if --gui flag or `pitu gui` subcommand was passed
    if cli.gui || matches!(cli.command, Some(Commands::Gui)) {
        return run_gui();
    }

    // Check subcommands that exit immediately
    match cli.command {
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "pitu", &mut io::stdout());
            return Ok(());
        }
        Some(Commands::Info) => {
            show_info_screen();
            return Ok(());
        }
        Some(Commands::Manual) => {
            show_manual_screen();
            return Ok(());
        }
        Some(Commands::InitConfig) => {
            return create_default_config_file();
        }
        Some(Commands::Sync { ref file, ref message }) => {
            let entry = create_snapshot(Path::new(file), message)?;
            print_success(
                &format!("Snapshot commit created! Hash: [{}]", entry.hash),
                false,
            );
            return Ok(());
        }
        Some(Commands::History { ref file }) => {
            let history = list_history(Path::new(file));
            if history.is_empty() {
                println!("\n  No version history found for file: {}", file);
            } else {
                println!("\n  📜 VERSION COMMIT HISTORY TIMELINE for: {}", file);
                println!("  ───────────────────────────────────────────────────────────");
                for entry in history {
                    println!(
                        "  ▫ [{}] {} - {}",
                        console::style(&entry.hash).cyan().bold(),
                        console::style(&entry.message).green(),
                        console::style(format!("{}s ago", entry.timestamp_sec)).dim()
                    );
                }
                println!();
            }
            return Ok(());
        }
        Some(Commands::Inspect { ref file }) => {
            return render_image_inspector(Path::new(file));
        }
        Some(Commands::Preview { ref file, heatmap }) => {
            return render_preview_cmd(Path::new(file), heatmap);
        }
        Some(Commands::Diff { ref input }) => {
            let sample_args = ProcessArgs {
                smart_crop: Some("16:9".to_string()),
                watermark_text: Some("© Pitu Diff".to_string()),
                ..Default::default()
            };
            let pipeline = Pipeline::from_process_args(&sample_args)?;
            return render_diff_cmd(Path::new(input), &pipeline);
        }
        Some(Commands::Preset { ref name, ref input }) => {
            let config = load_config();
            if let Some(preset) = config.presets.get(name) {
                let process_args = preset.to_process_args(input.clone());
                let pipeline = Pipeline::from_process_args(&process_args)?;
                let inputs = expand_input_paths(&[input.clone()]);
                let batch_opts = BatchOptions {
                    output: cli.output,
                    format: cli.format,
                    quality: cli.quality,
                    prefix: cli.prefix,
                    suffix: cli.suffix,
                    jobs: cli.jobs,
                    silent: cli.silent,
                    json: cli.json,
                    dry_run: cli.dry_run,
                    overwrite: cli.overwrite,
                };
                let report = execute_batch(&pipeline, &inputs, &batch_opts);
                if cli.json {
                    print_json_report(&report);
                }
                return Ok(());
            } else {
                print_error(&format!("Preset '{}' not found in pitu.toml or built-in presets.", name), cli.silent);
                exit(1);
            }
        }
        Some(Commands::InstallLauncher) => {
            return install_global_launcher();
        }
        _ => {}
    }

    // Check if interactive wizard requested explicitly or when no args are provided
    if matches!(cli.command, Some(Commands::Interactive))
        || (cli.command.is_none() && cli.input.is_none())
    {
        return interactive::run_interactive_wizard();
    }

    print_banner(cli.silent);

    let (process_args, inputs_raw) = match cli.command {
        Some(Commands::Process(args)) => (args.clone(), vec![args.input]),
        Some(Commands::Convert(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                ..Default::default()
            },
            args.input,
        ),
        Some(Commands::Resize(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                resize: args.width.or(args.height).map(|_| {
                    format!(
                        "{}x{}",
                        args.width.map(|w| w.to_string()).unwrap_or_else(|| "-".into()),
                        args.height.map(|h| h.to_string()).unwrap_or_else(|| "-".into())
                    )
                }),
                resize_filter: args.filter,
                resize_mode: args.mode,
                ..Default::default()
            },
            args.input,
        ),
        Some(Commands::Crop(args)) => {
            let crop_val = if args.ratio.is_some() {
                None
            } else if args.width.is_some() || args.height.is_some() {
                Some(format!(
                    "{},{},{},{}",
                    args.x,
                    args.y,
                    args.width.unwrap_or(0),
                    args.height.unwrap_or(0)
                ))
            } else {
                None
            };
            (
                ProcessArgs {
                    input: args.input.first().cloned().unwrap_or_default(),
                    crop: crop_val,
                    smart_crop: args.ratio,
                    ..Default::default()
                },
                args.input,
            )
        }
        Some(Commands::SmartCrop(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                smart_crop: args.ratio.or_else(|| {
                    if let (Some(w), Some(h)) = (args.width, args.height) {
                        Some(format!("{}x{}", w, h))
                    } else {
                        None
                    }
                }),
                ..Default::default()
            },
            args.input,
        ),
        Some(Commands::Rotate(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                rotate: Some(args.degrees),
                flip_h: args.flip_h,
                flip_v: args.flip_v,
                ..Default::default()
            },
            args.input,
        ),
        Some(Commands::Watermark(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                watermark_text: args.text,
                watermark_image: args.image,
                watermark_anchor: args.anchor,
                watermark_opacity: args.opacity,
                watermark_scale: args.scale,
                ..Default::default()
            },
            args.input,
        ),
        Some(Commands::Filter(args)) => (
            ProcessArgs {
                input: args.input.first().cloned().unwrap_or_default(),
                grayscale: args.grayscale,
                sepia: args.sepia,
                invert: args.invert,
                brightness: args.brightness,
                contrast: args.contrast,
                blur: args.blur,
                sharpen: args.sharpen,
                ..Default::default()
            },
            args.input,
        ),
        None => {
            if let Some(ref inp) = cli.input {
                (
                    ProcessArgs {
                        input: inp.clone(),
                        ..Default::default()
                    },
                    vec![inp.clone()],
                )
            } else {
                unreachable!()
            }
        }
        _ => unreachable!(),
    };

    let pipeline = Pipeline::from_process_args(&process_args)?;
    let inputs = expand_input_paths(&inputs_raw);

    if inputs.is_empty() {
        print_error("No matching input images found.", cli.silent);
        exit(1);
    }

    let batch_opts = BatchOptions {
        output: cli.output,
        format: cli.format,
        quality: cli.quality,
        prefix: cli.prefix,
        suffix: cli.suffix,
        jobs: cli.jobs,
        silent: cli.silent,
        json: cli.json,
        dry_run: cli.dry_run,
        overwrite: cli.overwrite,
    };

    let report = execute_batch(&pipeline, &inputs, &batch_opts);

    if cli.json {
        print_json_report(&report);
    } else {
        print_success(
            &format!(
                "Processed {} file(s): {} successful, {} failed in {}ms.",
                report.total_files, report.successful, report.failed, report.duration_ms
            ),
            cli.silent,
        );
    }

    if report.failed > 0 {
        exit(1);
    }

    Ok(())
}

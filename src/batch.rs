use crate::cli::ImageFormatChoice;
use crate::operations::{convert, Pipeline};
use crate::utils::{resolve_output_path, ProcessItemResult, ProcessReport};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct BatchOptions {
    pub output: Option<PathBuf>,
    pub format: Option<ImageFormatChoice>,
    pub quality: u8,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub jobs: Option<usize>,
    pub silent: bool,
    pub json: bool,
    pub dry_run: bool,
    pub overwrite: bool,
}

/// Resolve input patterns/paths into a list of existing image paths
pub fn expand_input_paths<S: AsRef<str>>(patterns: &[S]) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for pattern in patterns {
        let pat_str = pattern.as_ref();
        let path_obj = Path::new(pat_str);

        if path_obj.is_file() {
            paths.push(path_obj.to_path_buf());
            continue;
        }

        if path_obj.is_dir() {
            // Expand directory recursively
            let dir_pattern = format!("{}/**/*.{}:{}", pat_str, "png,jpg,jpeg,webp,bmp,gif,tiff", "");
            if let Ok(entries) = glob(&dir_pattern) {
                for entry in entries.flatten() {
                    if is_supported_image(&entry) {
                        paths.push(entry);
                    }
                }
            }
            continue;
        }

        // Try glob expansion
        if let Ok(entries) = glob(pat_str) {
            let mut matched = false;
            for entry in entries.flatten() {
                if entry.is_file() && is_supported_image(&entry) {
                    paths.push(entry);
                    matched = true;
                }
            }
            if matched {
                continue;
            }
        }

        // Fallback: direct path even if missing (for clear error reporting)
        paths.push(path_obj.to_path_buf());
    }

    paths.sort();
    paths.dedup();
    paths
}

fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_str.as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" | "tif" | "ico"
        )
    } else {
        false
    }
}

/// Execute pipeline on multiple files in parallel
pub fn execute_batch(
    pipeline: &Pipeline,
    inputs: &[PathBuf],
    opts: &BatchOptions,
) -> ProcessReport {
    let start_time = Instant::now();
    let total_files = inputs.len();

    if let Some(n_threads) = opts.jobs {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads)
            .build_global();
    }

    let progress_bar = if !opts.silent && !opts.json && total_files > 1 {
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let success_counter = Arc::new(AtomicUsize::new(0));
    let fail_counter = Arc::new(AtomicUsize::new(0));

    let items: Vec<ProcessItemResult> = inputs
        .par_iter()
        .map(|input_path| {
            let item_start = Instant::now();
            let output_path = resolve_output_path(
                input_path,
                opts.output.as_deref(),
                opts.format,
                opts.prefix.as_deref(),
                opts.suffix.as_deref(),
            );

            if opts.dry_run {
                if let Some(ref pb) = progress_bar {
                    pb.inc(1);
                }
                return ProcessItemResult {
                    input_path: input_path.display().to_string(),
                    output_path: Some(output_path.display().to_string()),
                    success: true,
                    error: None,
                    duration_ms: item_start.elapsed().as_millis(),
                };
            }

            let result = process_single_file(pipeline, input_path, &output_path, opts);
            let elapsed = item_start.elapsed().as_millis();

            if let Some(ref pb) = progress_bar {
                pb.inc(1);
            }

            match result {
                Ok(_) => {
                    success_counter.fetch_add(1, Ordering::Relaxed);
                    ProcessItemResult {
                        input_path: input_path.display().to_string(),
                        output_path: Some(output_path.display().to_string()),
                        success: true,
                        error: None,
                        duration_ms: elapsed,
                    }
                }
                Err(err) => {
                    fail_counter.fetch_add(1, Ordering::Relaxed);
                    ProcessItemResult {
                        input_path: input_path.display().to_string(),
                        output_path: Some(output_path.display().to_string()),
                        success: false,
                        error: Some(err.to_string()),
                        duration_ms: elapsed,
                    }
                }
            }
        })
        .collect();

    if let Some(pb) = progress_bar {
        pb.finish_with_message("Batch processing complete.");
    }

    let duration_ms = start_time.elapsed().as_millis();
    let successful = success_counter.load(Ordering::Relaxed);
    let failed = fail_counter.load(Ordering::Relaxed);

    let report = ProcessReport {
        total_files,
        successful,
        failed,
        duration_ms,
        items,
    };

    if !opts.silent && !opts.json {
        for item in &report.items {
            if !item.success {
                if let Some(ref err) = item.error {
                    crate::utils::print_error(&format!("Failed file '{}': {}", item.input_path, err), false);
                }
            }
        }
    }

    report
}

fn process_single_file(
    pipeline: &Pipeline,
    input_path: &Path,
    output_path: &Path,
    opts: &BatchOptions,
) -> anyhow::Result<()> {
    let read_res = crate::operations::universal_reader::load_universal_image(input_path)?;
    let processed = pipeline.execute(&read_res.image)?;

    let target_format = opts
        .format
        .or_else(|| detect_format_from_path(output_path))
        .unwrap_or(ImageFormatChoice::Png);

    convert::convert_format(&processed, output_path, target_format, opts.quality)?;
    Ok(())
}

fn detect_format_from_path(path: &Path) -> Option<ImageFormatChoice> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some(ImageFormatChoice::Jpeg),
        "png" => Some(ImageFormatChoice::Png),
        "webp" => Some(ImageFormatChoice::Webp),
        "gif" => Some(ImageFormatChoice::Gif),
        "bmp" => Some(ImageFormatChoice::Bmp),
        "tiff" | "tif" => Some(ImageFormatChoice::Tiff),
        "ico" => Some(ImageFormatChoice::Ico),
        _ => None,
    }
}

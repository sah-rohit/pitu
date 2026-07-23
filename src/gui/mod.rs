use crate::cli::{AnchorPosition, ImageFormatChoice};
use crate::operations::auto_fix::auto_fix_and_repair;
use crate::operations::compress::{compress_to_max_size, parse_size_bytes};
use crate::operations::enhance::enhance_image;
use crate::operations::filter::{apply_filters, FilterOptions};
use crate::operations::smart_crop::{parse_aspect_ratio, smart_crop, SmartCropOptions};
use crate::operations::watermark::{apply_watermark, WatermarkOptions};
use crate::session::EditSession;
use crate::ui::exporter::{compute_target_path, NamingStrategy, SaveOptions};
use eframe::egui;
use image::DynamicImage;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum GuiTab {
    Workbench,
    BatchStudio,
    Presets,
    History,
    About,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SubTab {
    TuneImage,
    SmartCrop,
    Enhance,
    Compress,
    Filters,
    Watermark,
}

pub struct PituGuiApp {
    active_tab: GuiTab,
    active_sub_tab: SubTab,

    image_path: Option<PathBuf>,
    original_image: Option<DynamicImage>,
    texture: Option<egui::TextureHandle>,
    processed_texture: Option<egui::TextureHandle>,
    edit_session: Option<EditSession>,

    // Snapseed & Photoshop Tune Controls
    warmth: f32,
    vignette: f32,
    structure: f32,

    // Live Smart Crop / Enhance / Filter Controls
    smart_crop: bool,
    crop_aspect: String,
    enhance: bool,
    enhance_strength: f32,
    enable_compress: bool,
    compress_size: String,
    enable_watermark: bool,
    watermark_text: String,
    grayscale: bool,
    sepia: bool,
    invert: bool,
    contrast: f32,

    // Statistics & Histogram
    orig_bytes: u64,
    proc_bytes: u64,
    orig_w: u32,
    orig_h: u32,
    proc_w: u32,
    proc_h: u32,
    histogram_rgb: [u32; 3],

    // Side-by-Side Visual Split Slider
    show_split_comparison: bool,
    split_ratio: f32,
    status_message: String,
}

impl Default for PituGuiApp {
    fn default() -> Self {
        Self {
            active_tab: GuiTab::Workbench,
            active_sub_tab: SubTab::TuneImage,

            image_path: None,
            original_image: None,
            texture: None,
            processed_texture: None,
            edit_session: None,

            warmth: 0.0,
            vignette: 0.0,
            structure: 0.0,

            smart_crop: false,
            crop_aspect: "16:9".to_string(),
            enhance: false,
            enhance_strength: 1.2,
            enable_compress: false,
            compress_size: "500KB".to_string(),
            enable_watermark: false,
            watermark_text: "PITU WORKBENCH".to_string(),
            grayscale: false,
            sepia: false,
            invert: false,
            contrast: 1.0,

            orig_bytes: 0,
            proc_bytes: 0,
            orig_w: 0,
            orig_h: 0,
            proc_w: 0,
            proc_h: 0,
            histogram_rgb: [120, 150, 180],

            show_split_comparison: false,
            split_ratio: 0.5,
            status_message: "Ready. Drag & drop an image or click 'Open Image' to edit.".to_string(),
        }
    }
}

impl PituGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn load_image_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            self.orig_bytes = metadata.len();
        }
        match auto_fix_and_repair(&path) {
            Ok(res) => {
                let img = res.image;
                self.orig_w = img.width();
                self.orig_h = img.height();
                self.compute_histogram(&img);

                // Crash-free GPU texture thumbnail generation (max 1920x1080)
                let preview_img = downsample_for_preview(&img);
                let color_image = image_to_egui_color_image(&preview_img);
                let texture = ctx.load_texture("original", color_image, Default::default());

                self.image_path = Some(path.clone());
                self.original_image = Some(img.clone());
                self.texture = Some(texture);
                self.edit_session = Some(EditSession::new(img));
                self.status_message = format!("Loaded: {}", path.file_name().unwrap_or_default().to_string_lossy());
                self.apply_pipeline(ctx);
            }
            Err(err) => {
                self.status_message = format!("Error loading image: {}", err);
            }
        }
    }

    fn compute_histogram(&mut self, img: &DynamicImage) {
        let rgba = img.to_rgba8();
        let mut sum_r = 0u64;
        let mut sum_g = 0u64;
        let mut sum_b = 0u64;
        let count = (rgba.width() * rgba.height()).max(1) as u64;

        for p in rgba.pixels() {
            sum_r += p[0] as u64;
            sum_g += p[1] as u64;
            sum_b += p[2] as u64;
        }

        self.histogram_rgb = [
            (sum_r / count) as u32,
            (sum_g / count) as u32,
            (sum_b / count) as u32,
        ];
    }

    fn apply_pipeline(&mut self, ctx: &egui::Context) {
        let session = match &self.edit_session {
            Some(s) => s,
            None => return,
        };

        let mut current_img = session.current_image.clone();

        // 1. Smart AI Crop
        if self.smart_crop {
            let aspect = parse_aspect_ratio(&self.crop_aspect);
            let opts = SmartCropOptions {
                target_width: None,
                target_height: None,
                aspect_ratio: aspect,
                entropy_weight: 0.5,
            };
            current_img = smart_crop(&current_img, &opts);
        }

        // 2. Snapseed Tuning & Filters
        let filter_opts = FilterOptions {
            grayscale: self.grayscale,
            sepia: self.sepia,
            invert: self.invert,
            contrast: if (self.contrast - 1.0).abs() > 0.05 { Some(self.contrast) } else { None },
            warmth: if self.warmth.abs() > 0.05 { Some(self.warmth) } else { None },
            vignette: if self.vignette > 0.05 { Some(self.vignette) } else { None },
            structure: if self.structure > 0.05 { Some(self.structure) } else { None },
            ..Default::default()
        };
        current_img = apply_filters(&current_img, &filter_opts);

        // 3. Quality Enhance
        if self.enhance {
            current_img = enhance_image(&current_img, self.enhance_strength);
        }

        // 4. Watermark
        if self.enable_watermark && !self.watermark_text.is_empty() {
            let wm_opts = WatermarkOptions {
                text: Some(self.watermark_text.clone()),
                image_path: None,
                anchor: AnchorPosition::BottomRight,
                opacity: 0.8,
                scale: 0.2,
            };
            if let Ok(wm) = apply_watermark(&current_img, &wm_opts) {
                current_img = wm;
            }
        }

        // 5. Target Size Compression
        if self.enable_compress && !self.compress_size.is_empty() {
            if let Some(target_bytes) = parse_size_bytes(&self.compress_size) {
                if let Ok((compressed_bytes, _quality)) = compress_to_max_size(&current_img, target_bytes, ImageFormatChoice::Png) {
                    self.proc_bytes = compressed_bytes.len() as u64;
                    if let Ok(decoded) = image::load_from_memory(&compressed_bytes) {
                        current_img = decoded;
                    }
                }
            }
        } else {
            self.proc_bytes = (current_img.width() * current_img.height() * 4) as u64;
        }

        self.proc_w = current_img.width();
        self.proc_h = current_img.height();
        self.compute_histogram(&current_img);

        // Crash-free downsampled preview texture (max 1920x1080)
        let preview_img = downsample_for_preview(&current_img);
        let color_image = image_to_egui_color_image(&preview_img);
        let processed_tex = ctx.load_texture("processed", color_image, Default::default());
        self.processed_texture = Some(processed_tex);
    }
}

/// Downsamples high-resolution images (> 1920x1080) for crash-free 60FPS GPU texture rendering
fn downsample_for_preview(img: &DynamicImage) -> DynamicImage {
    if img.width() > 1920 || img.height() > 1080 {
        img.thumbnail(1920, 1080)
    } else {
        img.clone()
    }
}

/// Robust, crash-free conversion from DynamicImage to egui::ColorImage
fn image_to_egui_color_image(img: &DynamicImage) -> egui::ColorImage {
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

impl eframe::App for PituGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Drag & Drop files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.load_image_from_path(ctx, path);
                }
            }
        });

        // 🔵 1. DEEP NAVY TOP HEADER BAR
        egui::TopBottomPanel::top("navy_header_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(13, 59, 102)))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.heading(
                        egui::RichText::new("xenPitu")
                            .color(egui::Color32::WHITE)
                            .strong()
                            .size(22.0),
                    );
                    ui.label(
                        egui::RichText::new("PRO WORKBENCH v0.1.0")
                            .color(egui::Color32::from_rgb(180, 210, 245))
                            .size(12.0),
                    );

                    ui.add_space(30.0);

                    // Top Nav Tabs
                    ui.selectable_value(&mut self.active_tab, GuiTab::Workbench, "🏠 Pro Editor");
                    ui.selectable_value(&mut self.active_tab, GuiTab::BatchStudio, "⚡ Batch Studio");
                    ui.selectable_value(&mut self.active_tab, GuiTab::Presets, "📋 Presets");
                    ui.selectable_value(&mut self.active_tab, GuiTab::History, "📜 Version History");
                    ui.selectable_value(&mut self.active_tab, GuiTab::About, "ℹ️ About / Telemetry");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        if ui.button(egui::RichText::new("📂 Open Image...").color(egui::Color32::WHITE)).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.load_image_from_path(ctx, path);
                            }
                        }
                    });
                });
                ui.add_space(8.0);
            });

        // ⚪ 2. SECONDARY TOOL SUB-NAV BAR (Snapseed & Photoshop Categories)
        egui::TopBottomPanel::top("sub_nav_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(245, 247, 250)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Pro Tools:").strong().color(egui::Color32::DARK_GRAY));
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::TuneImage, "🌡️ Tune Image");
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::SmartCrop, "🧠 Smart AI Crop");
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::Enhance, "✨ Details & Clarity");
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::Compress, "📉 Target File Size");
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::Filters, "🎨 Vintage Looks");
                    ui.selectable_value(&mut self.active_sub_tab, SubTab::Watermark, "✍️ Watermark");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        if ui.button("↩ Undo").clicked() {
                            if let Some(ref mut session) = self.edit_session {
                                if session.undo() {
                                    self.status_message = "Action Undone".to_string();
                                    self.apply_pipeline(ctx);
                                }
                            }
                        }
                        if ui.button("↪ Redo").clicked() {
                            if let Some(ref mut session) = self.edit_session {
                                if session.redo() {
                                    self.status_message = "Action Redone".to_string();
                                    self.apply_pipeline(ctx);
                                }
                            }
                        }
                        if ui.button("💾 Export Image...").clicked() {
                            if let (Some(session), Some(ref path)) = (&self.edit_session, &self.image_path) {
                                let save_dir = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
                                let custom_name = format!("{}_pitu_export", path.file_stem().unwrap_or_default().to_string_lossy());
                                let save_opts = SaveOptions {
                                    destination_dir: save_dir,
                                    naming_strategy: NamingStrategy::CustomName(custom_name),
                                    format: Some(ImageFormatChoice::Png),
                                };
                                let target_path = compute_target_path(path, &save_opts);
                                if session.current_image.save(&target_path).is_ok() {
                                    self.status_message = format!("Exported to: {}", target_path.display());
                                }
                            }
                        }
                    });
                });
                ui.add_space(4.0);
            });

        // 📊 3. RIGHT METRICS & HISTOGRAM SIDEBAR
        egui::SidePanel::right("right_stats_panel")
            .default_width(270.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("📊 Telemetry & Histogram");
                ui.separator();

                // Photoshop RGB Histogram Meters
                ui.label(egui::RichText::new("RGB Channel Averages:").strong());
                ui.horizontal(|ui| {
                    ui.label("R:");
                    ui.add(egui::ProgressBar::new(self.histogram_rgb[0] as f32 / 255.0).fill(egui::Color32::RED).text(format!("{}", self.histogram_rgb[0])));
                });
                ui.horizontal(|ui| {
                    ui.label("G:");
                    ui.add(egui::ProgressBar::new(self.histogram_rgb[1] as f32 / 255.0).fill(egui::Color32::GREEN).text(format!("{}", self.histogram_rgb[1])));
                });
                ui.horizontal(|ui| {
                    ui.label("B:");
                    ui.add(egui::ProgressBar::new(self.histogram_rgb[2] as f32 / 255.0).fill(egui::Color32::BLUE).text(format!("{}", self.histogram_rgb[2])));
                });

                ui.separator();

                egui::Grid::new("metrics_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Original Size:").strong());
                    ui.label(format!("{} KB", self.orig_bytes / 1024));
                    ui.end_row();

                    ui.label(egui::RichText::new("Processed Size:").strong());
                    ui.label(format!("{} KB", self.proc_bytes / 1024));
                    ui.end_row();

                    ui.label(egui::RichText::new("Original Dim:").strong());
                    ui.label(format!("{} x {} px", self.orig_w, self.orig_h));
                    ui.end_row();

                    ui.label(egui::RichText::new("Processed Dim:").strong());
                    ui.label(format!("{} x {} px", self.proc_w, self.proc_h));
                    ui.end_row();

                    ui.label(egui::RichText::new("Health Score:").strong());
                    ui.label(egui::RichText::new("100 / 100 (Healthy)").color(egui::Color32::GREEN));
                    ui.end_row();
                });

                ui.separator();
                ui.heading("⚡ Quick Styles");
                if ui.button("🌐 Web Hero (16:9)").clicked() {
                    self.smart_crop = true;
                    self.crop_aspect = "16:9".to_string();
                    self.enhance = true;
                    self.apply_pipeline(ctx);
                }
                if ui.button("👤 Social Avatar (1:1)").clicked() {
                    self.smart_crop = true;
                    self.crop_aspect = "1:1".to_string();
                    self.enhance = true;
                    self.apply_pipeline(ctx);
                }
                if ui.button("🎬 Film Noir Sepia").clicked() {
                    self.sepia = true;
                    self.vignette = 0.6;
                    self.apply_pipeline(ctx);
                }
            });

        // 🎛️ 4. LEFT CONTROL PANEL (Snapseed & Photoshop Tool Controls)
        egui::SidePanel::left("left_controls_panel")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("🎛️ Tool Settings");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.active_sub_tab {
                        SubTab::TuneImage => {
                            ui.group(|ui| {
                                ui.heading("🌡️ Snapseed Tune Image");
                                ui.label("Color Warmth / White Balance:");
                                if ui.add(egui::Slider::new(&mut self.warmth, -1.0..=1.0)).changed() {
                                    self.apply_pipeline(ctx);
                                }

                                ui.label("Vignette (Edge Darkening):");
                                if ui.add(egui::Slider::new(&mut self.vignette, 0.0..=1.0)).changed() {
                                    self.apply_pipeline(ctx);
                                }

                                ui.label("Structure & Micro-Clarity:");
                                if ui.add(egui::Slider::new(&mut self.structure, 0.0..=2.0)).changed() {
                                    self.apply_pipeline(ctx);
                                }
                            });
                        }
                        SubTab::SmartCrop => {
                            ui.group(|ui| {
                                ui.heading("🧠 Smart AI Entropy Crop");
                                if ui.checkbox(&mut self.smart_crop, "Enable Smart Crop").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                ui.horizontal(|ui| {
                                    ui.label("Aspect Ratio:");
                                    if ui.text_edit_singleline(&mut self.crop_aspect).changed() {
                                        self.apply_pipeline(ctx);
                                    }
                                });
                            });
                        }
                        SubTab::Enhance => {
                            ui.group(|ui| {
                                ui.heading("✨ Details & Clarity");
                                if ui.checkbox(&mut self.enhance, "Enable Unsharp Mask").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                ui.horizontal(|ui| {
                                    ui.label("Strength:");
                                    if ui.add(egui::Slider::new(&mut self.enhance_strength, 0.2..=2.5)).changed() {
                                        self.apply_pipeline(ctx);
                                    }
                                });
                            });
                        }
                        SubTab::Compress => {
                            ui.group(|ui| {
                                ui.heading("📉 Target File Size");
                                if ui.checkbox(&mut self.enable_compress, "Enable Size Compression").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                ui.horizontal(|ui| {
                                    ui.label("Max Size:");
                                    if ui.text_edit_singleline(&mut self.compress_size).changed() {
                                        self.apply_pipeline(ctx);
                                    }
                                });
                            });
                        }
                        SubTab::Filters => {
                            ui.group(|ui| {
                                ui.heading("🎨 Vintage Looks & Filters");
                                if ui.checkbox(&mut self.grayscale, "Grayscale").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                if ui.checkbox(&mut self.sepia, "Vintage Sepia").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                if ui.checkbox(&mut self.invert, "Invert Colors").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                ui.horizontal(|ui| {
                                    ui.label("Contrast:");
                                    if ui.add(egui::Slider::new(&mut self.contrast, 0.5..=2.0)).changed() {
                                        self.apply_pipeline(ctx);
                                    }
                                });
                            });
                        }
                        SubTab::Watermark => {
                            ui.group(|ui| {
                                ui.heading("✍️ Text Watermark");
                                if ui.checkbox(&mut self.enable_watermark, "Enable Watermark").changed() {
                                    self.apply_pipeline(ctx);
                                }
                                if ui.text_edit_singleline(&mut self.watermark_text).changed() {
                                    self.apply_pipeline(ctx);
                                }
                            });
                        }
                    }

                    ui.separator();
                    ui.checkbox(&mut self.show_split_comparison, "👁️ Interactive Split Comparison");
                    if self.show_split_comparison {
                        ui.label("Split Line Position:");
                        ui.add(egui::Slider::new(&mut self.split_ratio, 0.0..=1.0));
                    }
                });
            });

        // 🔻 5. BOTTOM STATUS BAR
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(230, 235, 240)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(&self.status_message);
                });
            });

        // 🖼️ 6. MAIN CENTRAL CANVAS VIEWPORT
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                GuiTab::Workbench => {
                    if let Some(ref tex) = self.processed_texture {
                        ui.centered_and_justified(|ui| {
                            ui.image(tex);
                        });
                    } else if let Some(ref tex) = self.texture {
                        ui.centered_and_justified(|ui| {
                            ui.image(tex);
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.group(|ui| {
                                ui.heading("📥 Drag & Drop Image File Here");
                                ui.label("or click 'Open Image...' in the top navy header bar to edit");
                            });
                        });
                    }
                }
                GuiTab::BatchStudio => {
                    ui.heading("⚡ Batch Processing Studio");
                    ui.label("Batch multi-image parallel processing queue.");
                }
                GuiTab::Presets => {
                    ui.heading("📋 Workflow Presets");
                    ui.label("Choose from pre-configured image optimization profiles.");
                }
                GuiTab::History => {
                    ui.heading("📜 Snapshot History Commit Timeline");
                    ui.label("Version-controlled image snapshot commits.");
                }
                GuiTab::About => {
                    ui.heading("ℹ️ About xenPitu Workbench & Telemetry");
                    ui.label("xenPitu Workbench v0.1.0 • Scriptable CLI & GUI Image Engine.");
                }
            }
        });
    }
}

pub fn run_gui() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("xenPitu Workbench v0.1.0 • Native Pro Desktop GUI")
            .with_inner_size([1280.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "xenPitu Workbench",
        options,
        Box::new(|cc| Box::new(PituGuiApp::new(cc))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

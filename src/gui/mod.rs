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

pub struct PituGuiApp {
    image_path: Option<PathBuf>,
    original_image: Option<DynamicImage>,
    texture: Option<egui::TextureHandle>,
    processed_texture: Option<egui::TextureHandle>,
    edit_session: Option<EditSession>,

    // Live Sliders & Controls State
    smart_crop: bool,
    crop_aspect: String,
    enhance: bool,
    enhance_strength: f32,
    enable_compress: bool,
    compress_size: String,
    enable_watermark: bool,
    watermark_text: String,
    grayscale: bool,
    contrast: f32,

    // Side-by-Side Visual Split Slider (0.0 to 1.0)
    split_ratio: f32,
    status_message: String,
}

impl Default for PituGuiApp {
    fn default() -> Self {
        Self {
            image_path: None,
            original_image: None,
            texture: None,
            processed_texture: None,
            edit_session: None,

            smart_crop: false,
            crop_aspect: "16:9".to_string(),
            enhance: false,
            enhance_strength: 1.2,
            enable_compress: false,
            compress_size: "500KB".to_string(),
            enable_watermark: false,
            watermark_text: "PITU WORKBENCH".to_string(),
            grayscale: false,
            contrast: 1.0,

            split_ratio: 0.5,
            status_message: "Drag & Drop an image file anywhere to begin...".to_string(),
        }
    }
}

impl PituGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn load_image_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match auto_fix_and_repair(&path) {
            Ok(res) => {
                let img = res.image;
                let color_image = image_to_egui_color_image(&img);
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

    fn apply_pipeline(&mut self, ctx: &egui::Context) {
        let session = match &self.edit_session {
            Some(s) => s,
            None => return,
        };

        let mut current_img = session.current_image.clone();

        // 1. Smart Crop
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

        // 2. Enhance
        if self.enhance {
            current_img = enhance_image(&current_img, self.enhance_strength);
        }

        // 3. Filters
        let filter_opts = FilterOptions {
            grayscale: self.grayscale,
            contrast: if (self.contrast - 1.0).abs() > 0.05 { Some(self.contrast) } else { None },
            ..Default::default()
        };
        current_img = apply_filters(&current_img, &filter_opts);

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

        // 5. Target Compression Simulation
        if self.enable_compress && !self.compress_size.is_empty() {
            if let Some(target_bytes) = parse_size_bytes(&self.compress_size) {
                if let Ok((compressed_bytes, _quality)) = compress_to_max_size(&current_img, target_bytes, ImageFormatChoice::Png) {
                    if let Ok(decoded) = image::load_from_memory(&compressed_bytes) {
                        current_img = decoded;
                    }
                }
            }
        }

        let color_image = image_to_egui_color_image(&current_img);
        let processed_tex = ctx.load_texture("processed", color_image, Default::default());
        self.processed_texture = Some(processed_tex);
    }
}

fn image_to_egui_color_image(img: &DynamicImage) -> egui::ColorImage {
    let rgba = img.to_rgba8();
    let size = [rgba.width() as _, rgba.height() as _];
    let pixels = rgba.as_flat_samples();
    egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
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

        // Top Control Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📷 PITU WORKBENCH • Desktop GUI");
                ui.separator();

                if ui.button("📂 Open Image...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.load_image_from_path(ctx, path);
                    }
                }

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

                ui.separator();
                ui.label("Preset Chips:");
                if ui.button("🌐 Web Hero").clicked() {
                    self.smart_crop = true;
                    self.crop_aspect = "16:9".to_string();
                    self.enhance = true;
                    self.apply_pipeline(ctx);
                }
                if ui.button("👤 Social Avatar").clicked() {
                    self.smart_crop = true;
                    self.crop_aspect = "1:1".to_string();
                    self.enhance = true;
                    self.apply_pipeline(ctx);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
        });

        // Left Control Panel
        egui::SidePanel::left("left_panel").resizable(true).default_width(280.0).show(ctx, |ui| {
            ui.heading("🎛️ Pipeline Controls");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Smart Crop Group
                ui.collapsing("🧠 Smart Entropy Crop", |ui| {
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

                ui.separator();

                // Enhancement Group
                ui.collapsing("✨ Quality Enhancement", |ui| {
                    if ui.checkbox(&mut self.enhance, "Adaptive Unsharp Mask & Pop").changed() {
                        self.apply_pipeline(ctx);
                    }
                    ui.horizontal(|ui| {
                        ui.label("Strength:");
                        if ui.add(egui::Slider::new(&mut self.enhance_strength, 0.2..=2.5)).changed() {
                            self.apply_pipeline(ctx);
                        }
                    });
                });

                ui.separator();

                // Target Size Compression
                ui.collapsing("📉 Target Size Compression", |ui| {
                    if ui.checkbox(&mut self.enable_compress, "Enable Size Targeting").changed() {
                        self.apply_pipeline(ctx);
                    }
                    ui.horizontal(|ui| {
                        ui.label("Max Size:");
                        if ui.text_edit_singleline(&mut self.compress_size).changed() {
                            self.apply_pipeline(ctx);
                        }
                    });
                });

                ui.separator();

                // Filters Group
                ui.collapsing("🎨 Visual Filters", |ui| {
                    if ui.checkbox(&mut self.grayscale, "Grayscale").changed() {
                        self.apply_pipeline(ctx);
                    }
                    ui.horizontal(|ui| {
                        ui.label("Contrast:");
                        if ui.add(egui::Slider::new(&mut self.contrast, 0.5..=2.0)).changed() {
                            self.apply_pipeline(ctx);
                        }
                    });
                });

                ui.separator();

                // Watermark Group
                ui.collapsing("✍️ Text Watermark", |ui| {
                    if ui.checkbox(&mut self.enable_watermark, "Enable Watermark").changed() {
                        self.apply_pipeline(ctx);
                    }
                    if ui.text_edit_singleline(&mut self.watermark_text).changed() {
                        self.apply_pipeline(ctx);
                    }
                });

                ui.separator();

                // Side-by-Side Split Ratio Slider
                ui.collapsing("👁️ Split Comparison", |ui| {
                    ui.label("Split Ratio:");
                    ui.add(egui::Slider::new(&mut self.split_ratio, 0.0..=1.0));
                });
            });
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
            });
        });

        // Main Center Viewport Panel
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        ui.label("or click 'Open Image...' in the top toolbar to begin editing");
                    });
                });
            }
        });
    }
}

pub fn run_gui() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pitu Workbench v0.1.0 • Native Desktop GUI")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pitu Workbench",
        options,
        Box::new(|cc| Box::new(PituGuiApp::new(cc))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

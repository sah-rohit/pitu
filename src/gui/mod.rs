use crate::cli::{AnchorPosition, ImageFormatChoice};
use crate::operations::auto_fix::auto_fix_and_repair;
use crate::operations::compress::{compress_to_max_size, parse_size_bytes};
use crate::operations::enhance::enhance_image;
use crate::operations::filter::{apply_filters, FilterOptions};
use crate::operations::smart_crop::{parse_aspect_ratio, smart_crop, SmartCropOptions};
use crate::operations::watermark::{apply_watermark, WatermarkOptions};
use crate::session::EditSession;
use eframe::egui;
use image::DynamicImage;
use std::path::PathBuf;

// ── Color Palette (Modern Dark UI) ──────────────────────────────────────────
const BG_DARK: egui::Color32     = egui::Color32::from_rgb(24, 24, 28);
const BG_PANEL: egui::Color32    = egui::Color32::from_rgb(32, 32, 38);
const BG_HEADER: egui::Color32   = egui::Color32::from_rgb(18, 18, 22);
const ACCENT: egui::Color32      = egui::Color32::from_rgb(99, 140, 255);
const ACCENT_DIM: egui::Color32  = egui::Color32::from_rgb(60, 90, 180);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 232, 240);
const TEXT_DIM: egui::Color32     = egui::Color32::from_rgb(140, 145, 160);
const SURFACE: egui::Color32     = egui::Color32::from_rgb(40, 42, 50);
const BORDER: egui::Color32      = egui::Color32::from_rgb(55, 58, 68);
const SUCCESS: egui::Color32     = egui::Color32::from_rgb(72, 199, 142);
const DANGER: egui::Color32      = egui::Color32::from_rgb(255, 99, 99);

// ── Tool Categories ─────────────────────────────────────────────────────────
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ToolCategory {
    TuneImage,
    Details,
    CropRotate,
    Filters,
    Effects,
    Watermark,
    Compress,
    Layers,
}

// ── View Mode ───────────────────────────────────────────────────────────────
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ViewMode {
    Processed,
    Original,
    SplitCompare,
}

// ── App State ───────────────────────────────────────────────────────────────
pub struct PituGuiApp {
    tool: ToolCategory,
    view_mode: ViewMode,

    image_path: Option<PathBuf>,
    original_image: Option<DynamicImage>,
    original_texture: Option<egui::TextureHandle>,
    processed_image: Option<DynamicImage>,
    processed_texture: Option<egui::TextureHandle>,
    edit_session: Option<EditSession>,

    // Tune Image
    warmth: f32,
    brightness: i32,
    contrast: f32,
    vignette: f32,
    structure: f32,

    // Crop
    smart_crop: bool,
    crop_aspect: String,

    // Enhance
    enhance: bool,
    enhance_strength: f32,

    // Filters
    grayscale: bool,
    sepia: bool,
    invert: bool,

    // Effects
    hdr_scape: bool,
    glamour_glow: bool,
    haze_removal: bool,
    frame_width: u32,

    // Watermark
    enable_watermark: bool,
    watermark_text: String,

    // Compress
    enable_compress: bool,
    compress_size: String,

    // Stats
    orig_bytes: u64,
    proc_bytes: u64,
    orig_w: u32,
    orig_h: u32,
    proc_w: u32,
    proc_h: u32,
    histogram_r: [u32; 256],
    histogram_g: [u32; 256],
    histogram_b: [u32; 256],

    // Split
    split_pos: f32,

    status: String,
    needs_reprocess: bool,
}

impl Default for PituGuiApp {
    fn default() -> Self {
        Self {
            tool: ToolCategory::TuneImage,
            view_mode: ViewMode::Processed,

            image_path: None,
            original_image: None,
            original_texture: None,
            processed_image: None,
            processed_texture: None,
            edit_session: None,

            warmth: 0.0,
            brightness: 0,
            contrast: 0.0,
            vignette: 0.0,
            structure: 0.0,

            smart_crop: false,
            crop_aspect: "16:9".into(),

            enhance: false,
            enhance_strength: 1.2,

            grayscale: false,
            sepia: false,
            invert: false,

            hdr_scape: false,
            glamour_glow: false,
            haze_removal: false,
            frame_width: 0,

            enable_watermark: false,
            watermark_text: "Pitu".into(),

            enable_compress: false,
            compress_size: "500KB".into(),

            orig_bytes: 0,
            proc_bytes: 0,
            orig_w: 0,
            orig_h: 0,
            proc_w: 0,
            proc_h: 0,
            histogram_r: [0; 256],
            histogram_g: [0; 256],
            histogram_b: [0; 256],

            split_pos: 0.5,

            status: "Ready — Open an image to begin editing".into(),
            needs_reprocess: false,
        }
    }
}

impl PituGuiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        configure_style(&cc.egui_ctx);
        Self::default()
    }

    fn load_image(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.status = format!("Loading {}...", path.file_name().unwrap_or_default().to_string_lossy());
        if let Ok(meta) = std::fs::metadata(&path) {
            self.orig_bytes = meta.len();
        }
        match auto_fix_and_repair(&path) {
            Ok(res) => {
                let img = res.image;
                self.orig_w = img.width();
                self.orig_h = img.height();

                let preview = downsample(&img);
                let tex = ctx.load_texture("original", to_color_image(&preview), Default::default());
                self.original_texture = Some(tex);
                self.original_image = Some(img.clone());
                self.image_path = Some(path.clone());
                self.edit_session = Some(EditSession::new(img));

                self.reset_controls();
                self.run_pipeline(ctx);
                self.status = format!("Loaded: {}", path.file_name().unwrap_or_default().to_string_lossy());
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn reset_controls(&mut self) {
        self.warmth = 0.0;
        self.brightness = 0;
        self.contrast = 0.0;
        self.vignette = 0.0;
        self.structure = 0.0;
        self.smart_crop = false;
        self.enhance = false;
        self.grayscale = false;
        self.sepia = false;
        self.invert = false;
        self.hdr_scape = false;
        self.glamour_glow = false;
        self.haze_removal = false;
        self.frame_width = 0;
        self.enable_watermark = false;
        self.enable_compress = false;
    }

    fn run_pipeline(&mut self, ctx: &egui::Context) {
        let base_img = match &self.edit_session {
            Some(s) => s.initial_image.clone(),
            None => return,
        };

        let mut img = base_img;

        // 1. Smart Crop
        if self.smart_crop {
            let aspect = parse_aspect_ratio(&self.crop_aspect);
            img = smart_crop(&img, &SmartCropOptions {
                target_width: None,
                target_height: None,
                aspect_ratio: aspect,
                entropy_weight: 0.5,
            });
        }

        // 2. Filters & Tune
        let fopts = FilterOptions {
            grayscale: self.grayscale,
            sepia: self.sepia,
            invert: self.invert,
            brightness: if self.brightness != 0 { Some(self.brightness) } else { None },
            contrast: if self.contrast.abs() > 0.05 { Some(self.contrast) } else { None },
            warmth: if self.warmth.abs() > 0.05 { Some(self.warmth) } else { None },
            vignette: if self.vignette > 0.05 { Some(self.vignette) } else { None },
            structure: if self.structure > 0.05 { Some(self.structure) } else { None },
            hdr_scape: self.hdr_scape,
            glamour_glow: self.glamour_glow,
            haze_removal: self.haze_removal,
            frame_width: if self.frame_width > 0 { Some(self.frame_width) } else { None },
            ..Default::default()
        };
        img = apply_filters(&img, &fopts);

        // 3. Enhance
        if self.enhance {
            img = enhance_image(&img, self.enhance_strength);
        }

        // 4. Watermark
        if self.enable_watermark && !self.watermark_text.is_empty() {
            if let Ok(wm) = apply_watermark(&img, &WatermarkOptions {
                text: Some(self.watermark_text.clone()),
                image_path: None,
                anchor: AnchorPosition::BottomRight,
                opacity: 0.8,
                scale: 0.2,
            }) {
                img = wm;
            }
        }

        // 5. Compress
        if self.enable_compress && !self.compress_size.is_empty() {
            if let Some(target) = parse_size_bytes(&self.compress_size) {
                if let Ok((bytes, _)) = compress_to_max_size(&img, target, ImageFormatChoice::Png) {
                    self.proc_bytes = bytes.len() as u64;
                    if let Ok(decoded) = image::load_from_memory(&bytes) {
                        img = decoded;
                    }
                }
            }
        } else {
            self.proc_bytes = (img.width() * img.height() * 4) as u64;
        }

        self.proc_w = img.width();
        self.proc_h = img.height();
        self.compute_histogram(&img);

        // Store processed image for export
        self.processed_image = Some(img.clone());

        // Update session current_image so undo/redo works
        if let Some(ref mut session) = self.edit_session {
            session.current_image = img.clone();
        }

        let preview = downsample(&img);
        let tex = ctx.load_texture("processed", to_color_image(&preview), Default::default());
        self.processed_texture = Some(tex);
        self.needs_reprocess = false;
    }

    fn commit_and_reprocess(&mut self, ctx: &egui::Context, desc: &str) {
        // Push undo state before reprocessing
        if let Some(ref mut session) = self.edit_session {
            if let Some(ref processed) = self.processed_image {
                session.apply_action(processed.clone(), desc.to_string());
            }
        }
        self.run_pipeline(ctx);
    }

    fn do_undo(&mut self, ctx: &egui::Context) {
        if let Some(ref mut session) = self.edit_session {
            if session.undo() {
                // Restore the initial_image to the undo'd state
                session.initial_image = session.current_image.clone();
                self.original_image = Some(session.current_image.clone());
                let preview = downsample(&session.current_image);
                self.original_texture = Some(ctx.load_texture("original", to_color_image(&preview), Default::default()));
                self.reset_controls();
                self.run_pipeline(ctx);
                self.status = "Undo successful".into();
            }
        }
    }

    fn do_redo(&mut self, ctx: &egui::Context) {
        if let Some(ref mut session) = self.edit_session {
            if session.redo() {
                session.initial_image = session.current_image.clone();
                self.original_image = Some(session.current_image.clone());
                let preview = downsample(&session.current_image);
                self.original_texture = Some(ctx.load_texture("original", to_color_image(&preview), Default::default()));
                self.reset_controls();
                self.run_pipeline(ctx);
                self.status = "Redo successful".into();
            }
        }
    }

    fn do_export(&mut self) {
        let img = match &self.processed_image {
            Some(i) => i,
            None => {
                self.status = "Nothing to export — open an image first".into();
                return;
            }
        };

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export Processed Image")
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("WebP", &["webp"])
            .add_filter("BMP", &["bmp"])
            .save_file()
        {
            match img.save(&path) {
                Ok(()) => self.status = format!("Exported to {}", path.display()),
                Err(e) => self.status = format!("Export failed: {}", e),
            }
        }
    }

    fn compute_histogram(&mut self, img: &DynamicImage) {
        self.histogram_r = [0; 256];
        self.histogram_g = [0; 256];
        self.histogram_b = [0; 256];
        // Sample a downscaled version for speed
        let small = img.thumbnail(256, 256).to_rgba8();
        for p in small.pixels() {
            self.histogram_r[p[0] as usize] += 1;
            self.histogram_g[p[1] as usize] += 1;
            self.histogram_b[p[2] as usize] += 1;
        }
    }

    fn draw_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(BG_HEADER).inner_margin(egui::Margin::symmetric(16.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Pitu").size(20.0).strong().color(ACCENT));
                    ui.label(egui::RichText::new("Image Editor").size(12.0).color(TEXT_DIM));

                    ui.add_space(24.0);

                    // View mode toggle
                    let btn = |ui: &mut egui::Ui, label: &str, mode: ViewMode, current: &mut ViewMode| {
                        let selected = *current == mode;
                        let text = egui::RichText::new(label).size(11.0)
                            .color(if selected { ACCENT } else { TEXT_DIM });
                        if ui.add(egui::Button::new(text)
                            .fill(if selected { SURFACE } else { egui::Color32::TRANSPARENT })
                            .rounding(4.0)
                        ).clicked() {
                            *current = mode;
                        }
                    };
                    btn(ui, "Processed", ViewMode::Processed, &mut self.view_mode);
                    btn(ui, "Original", ViewMode::Original, &mut self.view_mode);
                    btn(ui, "Split Compare", ViewMode::SplitCompare, &mut self.view_mode);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Export
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Export").size(11.0).color(BG_DARK)
                        ).fill(ACCENT).rounding(4.0)).clicked() {
                            self.do_export();
                        }

                        ui.add_space(6.0);

                        // Redo
                        let can_redo = self.edit_session.as_ref().map_or(false, |s| s.can_redo());
                        if ui.add_enabled(can_redo, egui::Button::new(
                            egui::RichText::new("↪ Redo").size(11.0).color(TEXT_DIM)
                        ).fill(SURFACE).rounding(4.0)).clicked() {
                            self.do_redo(ctx);
                        }

                        // Undo
                        let can_undo = self.edit_session.as_ref().map_or(false, |s| s.can_undo());
                        if ui.add_enabled(can_undo, egui::Button::new(
                            egui::RichText::new("↩ Undo").size(11.0).color(TEXT_DIM)
                        ).fill(SURFACE).rounding(4.0)).clicked() {
                            self.do_undo(ctx);
                        }

                        // Open
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Open").size(11.0).color(TEXT_PRIMARY)
                        ).fill(SURFACE).rounding(4.0)).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png","jpg","jpeg","webp","bmp","gif","tiff","ico"])
                                .pick_file()
                            {
                                self.load_image(ctx, path);
                            }
                        }
                    });
                });
            });
    }

    fn draw_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(BG_HEADER).inner_margin(egui::Margin::symmetric(12.0, 4.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.status).size(10.0).color(TEXT_DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.orig_w > 0 {
                            ui.label(egui::RichText::new(
                                format!("{}×{} → {}×{}  |  {} KB → {} KB",
                                    self.orig_w, self.orig_h, self.proc_w, self.proc_h,
                                    self.orig_bytes / 1024, self.proc_bytes / 1024)
                            ).size(10.0).color(TEXT_DIM));
                        }
                    });
                });
            });
    }

    fn draw_tool_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("tools")
            .default_width(240.0)
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::same(12.0))
                .stroke(egui::Stroke::new(1.0_f32, BORDER)))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("TOOLS").size(10.0).strong().color(TEXT_DIM));
                ui.add_space(8.0);

                let tools = [
                    (ToolCategory::TuneImage,  "Tune Image"),
                    (ToolCategory::Details,    "Details & Clarity"),
                    (ToolCategory::CropRotate, "Crop & Rotate"),
                    (ToolCategory::Filters,    "Filters"),
                    (ToolCategory::Effects,    "Effects"),
                    (ToolCategory::Watermark,  "Watermark"),
                    (ToolCategory::Compress,   "Compress"),
                    (ToolCategory::Layers,     "Layer Stack"),
                ];

                for (cat, label) in tools {
                    let selected = self.tool == cat;
                    let text = egui::RichText::new(label).size(12.0)
                        .color(if selected { ACCENT } else { TEXT_PRIMARY });
                    let btn = egui::Button::new(text)
                        .fill(if selected { SURFACE } else { egui::Color32::TRANSPARENT })
                        .rounding(6.0);
                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                        self.tool = cat;
                    }
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_tool_controls(ui, ctx);
                });
            });
    }

    fn draw_tool_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut changed = false;

        match self.tool {
            ToolCategory::TuneImage => {
                ui.label(egui::RichText::new("Tune Image").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                ui.label(egui::RichText::new("Brightness").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.brightness, -100..=100)).changed();

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Contrast").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.contrast, -50.0..=50.0)).changed();

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Warmth").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.warmth, -1.0..=1.0)).changed();

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Vignette").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.vignette, 0.0..=1.5)).changed();
            }

            ToolCategory::Details => {
                ui.label(egui::RichText::new("Details & Clarity").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                ui.label(egui::RichText::new("Structure").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.structure, 0.0..=3.0)).changed();

                ui.add_space(4.0);
                changed |= ui.checkbox(&mut self.enhance, "Sharpen / Enhance").changed();
                if self.enhance {
                    ui.label(egui::RichText::new("Strength").size(10.0).color(TEXT_DIM));
                    changed |= ui.add(egui::Slider::new(&mut self.enhance_strength, 0.2..=3.0)).changed();
                }
            }

            ToolCategory::CropRotate => {
                ui.label(egui::RichText::new("Crop & Rotate").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                changed |= ui.checkbox(&mut self.smart_crop, "Smart AI Crop").changed();
                if self.smart_crop {
                    ui.label(egui::RichText::new("Aspect Ratio").size(10.0).color(TEXT_DIM));
                    ui.horizontal(|ui| {
                        for ratio in ["16:9", "4:3", "1:1", "3:2", "9:16"] {
                            let selected = self.crop_aspect == ratio;
                            let text = egui::RichText::new(ratio).size(10.0)
                                .color(if selected { ACCENT } else { TEXT_DIM });
                            if ui.add(egui::Button::new(text)
                                .fill(if selected { SURFACE } else { egui::Color32::TRANSPARENT })
                                .rounding(4.0)
                            ).clicked() {
                                self.crop_aspect = ratio.to_string();
                                changed = true;
                            }
                        }
                    });
                }
            }

            ToolCategory::Filters => {
                ui.label(egui::RichText::new("Filters").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                changed |= ui.checkbox(&mut self.grayscale, "Black & White").changed();
                changed |= ui.checkbox(&mut self.sepia, "Vintage Sepia").changed();
                changed |= ui.checkbox(&mut self.invert, "Invert / Negative").changed();
            }

            ToolCategory::Effects => {
                ui.label(egui::RichText::new("Effects").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                changed |= ui.checkbox(&mut self.hdr_scape, "HDR Scape").changed();
                changed |= ui.checkbox(&mut self.glamour_glow, "Glamour Glow").changed();
                changed |= ui.checkbox(&mut self.haze_removal, "Haze Removal").changed();

                ui.add_space(4.0);
                ui.label(egui::RichText::new("Frame Border").size(10.0).color(TEXT_DIM));
                changed |= ui.add(egui::Slider::new(&mut self.frame_width, 0..=80)).changed();
            }

            ToolCategory::Watermark => {
                ui.label(egui::RichText::new("Watermark").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                changed |= ui.checkbox(&mut self.enable_watermark, "Add Text Watermark").changed();
                if self.enable_watermark {
                    ui.label(egui::RichText::new("Text").size(10.0).color(TEXT_DIM));
                    changed |= ui.text_edit_singleline(&mut self.watermark_text).changed();
                }
            }

            ToolCategory::Compress => {
                ui.label(egui::RichText::new("Compress").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                changed |= ui.checkbox(&mut self.enable_compress, "Target File Size").changed();
                if self.enable_compress {
                    ui.label(egui::RichText::new("Max Size (e.g. 500KB, 2MB)").size(10.0).color(TEXT_DIM));
                    changed |= ui.text_edit_singleline(&mut self.compress_size).changed();
                }
            }

            ToolCategory::Layers => {
                ui.label(egui::RichText::new("Layer Stack").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                if let Some(ref mut session) = self.edit_session {
                    if session.layers.is_empty() {
                        ui.label(egui::RichText::new("No layers yet. Adjustments are applied directly.").size(10.0).color(TEXT_DIM));
                    } else {
                        let mut remove_idx: Option<usize> = None;
                        let mut toggle_idx: Option<usize> = None;

                        for (i, layer) in session.layers.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let icon = if layer.enabled { "●" } else { "○" };
                                let col = if layer.enabled { SUCCESS } else { TEXT_DIM };
                                if ui.add(egui::Button::new(egui::RichText::new(icon).color(col).size(12.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                ).clicked() {
                                    toggle_idx = Some(i);
                                }
                                ui.label(egui::RichText::new(&layer.name).size(11.0).color(TEXT_PRIMARY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(egui::RichText::new("✕").color(DANGER).size(10.0))
                                        .fill(egui::Color32::TRANSPARENT)
                                    ).clicked() {
                                        remove_idx = Some(i);
                                    }
                                });
                            });
                        }

                        if let Some(i) = toggle_idx {
                            session.toggle_layer_at(i);
                            changed = true;
                        }
                        if let Some(i) = remove_idx {
                            session.remove_layer_at(i);
                            changed = true;
                        }
                    }

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Undo History:").size(10.0).color(TEXT_DIM));
                    for entry in session.history_log.iter().rev().take(10) {
                        ui.label(egui::RichText::new(format!("  • {}", entry)).size(9.0).color(TEXT_DIM));
                    }
                } else {
                    ui.label(egui::RichText::new("Open an image first").size(10.0).color(TEXT_DIM));
                }
            }
        }

        if changed {
            self.run_pipeline(ctx);
        }

        // Apply / Commit button
        if self.tool != ToolCategory::Layers {
            ui.add_space(12.0);
            if ui.add_sized([ui.available_width(), 30.0],
                egui::Button::new(egui::RichText::new("Apply & Commit").size(11.0).color(BG_DARK))
                    .fill(ACCENT).rounding(6.0)
            ).clicked() {
                let desc = format!("{:?} adjustment", self.tool);
                self.commit_and_reprocess(ctx, &desc);
                self.status = "Changes committed to history".into();
            }
        }
    }

    fn draw_info_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("info")
            .default_width(200.0)
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::same(12.0))
                .stroke(egui::Stroke::new(1.0_f32, BORDER)))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("HISTOGRAM").size(10.0).strong().color(TEXT_DIM));
                ui.add_space(4.0);

                let hist_height = 60.0;
                let width = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(egui::vec2(width, hist_height), egui::Sense::hover());

                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 20, 24));

                let max_val = self.histogram_r.iter()
                    .chain(self.histogram_g.iter())
                    .chain(self.histogram_b.iter())
                    .copied().max().unwrap_or(1).max(1);

                let bar_w = width / 256.0;
                for i in 0..256 {
                    let x = rect.left() + i as f32 * bar_w;
                    let r_h = self.histogram_r[i] as f32 / max_val as f32 * hist_height;
                    let g_h = self.histogram_g[i] as f32 / max_val as f32 * hist_height;
                    let b_h = self.histogram_b[i] as f32 / max_val as f32 * hist_height;

                    painter.line_segment(
                        [egui::pos2(x, rect.bottom()), egui::pos2(x, rect.bottom() - r_h)],
                        egui::Stroke::new(bar_w.max(0.5), egui::Color32::from_rgba_premultiplied(255, 60, 60, 80)),
                    );
                    painter.line_segment(
                        [egui::pos2(x, rect.bottom()), egui::pos2(x, rect.bottom() - g_h)],
                        egui::Stroke::new(bar_w.max(0.5), egui::Color32::from_rgba_premultiplied(60, 255, 60, 80)),
                    );
                    painter.line_segment(
                        [egui::pos2(x, rect.bottom()), egui::pos2(x, rect.bottom() - b_h)],
                        egui::Stroke::new(bar_w.max(0.5), egui::Color32::from_rgba_premultiplied(60, 100, 255, 80)),
                    );
                }

                ui.add_space(8.0);
                ui.separator();

                ui.label(egui::RichText::new("IMAGE INFO").size(10.0).strong().color(TEXT_DIM));
                ui.add_space(4.0);

                let info = [
                    ("Original", format!("{}×{}", self.orig_w, self.orig_h)),
                    ("Processed", format!("{}×{}", self.proc_w, self.proc_h)),
                    ("File Size", format!("{} KB", self.orig_bytes / 1024)),
                    ("Output Size", format!("{} KB", self.proc_bytes / 1024)),
                ];

                for (label, value) in info {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(label).size(10.0).color(TEXT_DIM));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(value).size(10.0).color(TEXT_PRIMARY));
                        });
                    });
                }

                if self.view_mode == ViewMode::SplitCompare {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(egui::RichText::new("SPLIT POSITION").size(10.0).strong().color(TEXT_DIM));
                    ui.add(egui::Slider::new(&mut self.split_pos, 0.0..=1.0));
                }

                // Quick presets
                ui.add_space(8.0);
                ui.separator();
                ui.label(egui::RichText::new("QUICK PRESETS").size(10.0).strong().color(TEXT_DIM));
                ui.add_space(4.0);

                let presets: Vec<(&str, Box<dyn Fn(&mut PituGuiApp)>)> = vec![
                    ("Web Hero 16:9", Box::new(|app: &mut PituGuiApp| {
                        app.smart_crop = true; app.crop_aspect = "16:9".into(); app.enhance = true;
                    })),
                    ("Social 1:1", Box::new(|app: &mut PituGuiApp| {
                        app.smart_crop = true; app.crop_aspect = "1:1".into(); app.enhance = true;
                    })),
                    ("Film Noir", Box::new(|app: &mut PituGuiApp| {
                        app.sepia = true; app.vignette = 0.8; app.contrast = 15.0;
                    })),
                    ("HDR Pop", Box::new(|app: &mut PituGuiApp| {
                        app.hdr_scape = true; app.structure = 1.5;
                    })),
                ];

                for (name, apply_fn) in presets {
                    if ui.add_sized([ui.available_width(), 22.0],
                        egui::Button::new(egui::RichText::new(name).size(10.0).color(TEXT_PRIMARY))
                            .fill(SURFACE).rounding(4.0)
                    ).clicked() {
                        apply_fn(self);
                        self.run_pipeline(ctx);
                    }
                }
            });
    }

    fn draw_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARK))
            .show(ctx, |ui| {
                match self.view_mode {
                    ViewMode::Processed => {
                        if let Some(ref tex) = self.processed_texture {
                            self.draw_fitted_image(ui, tex);
                        } else {
                            self.draw_empty_state(ui);
                        }
                    }
                    ViewMode::Original => {
                        if let Some(ref tex) = self.original_texture {
                            self.draw_fitted_image(ui, tex);
                        } else {
                            self.draw_empty_state(ui);
                        }
                    }
                    ViewMode::SplitCompare => {
                        if let (Some(ref orig), Some(ref proc)) = (&self.original_texture, &self.processed_texture) {
                            self.draw_split_view(ui, orig, proc);
                        } else {
                            self.draw_empty_state(ui);
                        }
                    }
                }
            });
    }

    fn draw_fitted_image(&self, ui: &mut egui::Ui, tex: &egui::TextureHandle) {
        let available = ui.available_size();
        let tex_size = tex.size_vec2();
        let scale = (available.x / tex_size.x).min(available.y / tex_size.y).min(1.0);
        let size = tex_size * scale;
        let offset_x = (available.x - size.x) / 2.0;
        let offset_y = (available.y - size.y) / 2.0;

        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(
                ui.min_rect().min + egui::vec2(offset_x, offset_y),
                size,
            ),
            |ui| {
                ui.image(egui::load::SizedTexture::new(tex.id(), size));
            },
        );
    }

    fn draw_split_view(&self, ui: &mut egui::Ui, orig: &egui::TextureHandle, processed: &egui::TextureHandle) {
        let available = ui.available_size();
        let split_x = available.x * self.split_pos;

        // Left half: original
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(split_x, available.y)),
            |ui| {
                ui.set_clip_rect(egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(split_x, available.y)));
                self.draw_fitted_image(ui, orig);
            },
        );

        // Right half: processed
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(
                ui.min_rect().min + egui::vec2(split_x, 0.0),
                egui::vec2(available.x - split_x, available.y),
            ),
            |ui| {
                self.draw_fitted_image(ui, processed);
            },
        );

        // Divider line
        let painter = ui.painter();
        let x = ui.min_rect().left() + split_x;
        painter.line_segment(
            [egui::pos2(x, ui.min_rect().top()), egui::pos2(x, ui.min_rect().bottom())],
            egui::Stroke::new(2.0_f32, ACCENT),
        );
    }

    fn draw_empty_state(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.label(egui::RichText::new("Drop an image here").size(18.0).color(TEXT_DIM));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("or click Open in the toolbar").size(12.0).color(BORDER));
            });
        });
    }
}

// ── Helper Functions ────────────────────────────────────────────────────────

fn downsample(img: &DynamicImage) -> DynamicImage {
    if img.width() > 1920 || img.height() > 1080 {
        img.thumbnail(1920, 1080)
    } else {
        img.clone()
    }
}

fn to_color_image(img: &DynamicImage) -> egui::ColorImage {
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Use the built-in proportional font at higher quality
    fonts.families.entry(egui::FontFamily::Proportional)
        .or_default();
    ctx.set_fonts(fonts);
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Dark theme
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = BG_DARK;
    style.visuals.window_fill = BG_PANEL;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(16, 16, 20);
    style.visuals.faint_bg_color = SURFACE;

    style.visuals.widgets.noninteractive.bg_fill = SURFACE;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, TEXT_DIM);
    style.visuals.widgets.inactive.bg_fill = SURFACE;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, TEXT_PRIMARY);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 55, 70);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, ACCENT);
    style.visuals.widgets.active.bg_fill = ACCENT_DIM;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    style.visuals.selection.bg_fill = ACCENT_DIM;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT);

    // Crisp, not blurry
    style.visuals.window_rounding = egui::Rounding::same(6.0);
    style.visuals.menu_rounding = egui::Rounding::same(4.0);
    style.spacing.slider_width = 160.0;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);

    ctx.set_style(style);
}

// ── eframe App Trait ────────────────────────────────────────────────────────

impl eframe::App for PituGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag & drop
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.load_image(ctx, path);
                }
            }
        });

        self.draw_header(ctx);
        self.draw_status_bar(ctx);
        self.draw_tool_sidebar(ctx);
        self.draw_info_sidebar(ctx);
        self.draw_canvas(ctx);
    }
}

// ── Entry Point ─────────────────────────────────────────────────────────────

pub fn run_gui() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pitu — Image Editor")
            .with_inner_size([1360.0, 840.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pitu",
        options,
        Box::new(|cc| Box::new(PituGuiApp::new(cc))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

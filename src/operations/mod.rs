pub mod auto_fix;
pub mod compress;
pub mod convert;
pub mod crop;
pub mod enhance;
pub mod filter;
pub mod resize;
pub mod rotate;
pub mod smart_crop;
pub mod universal_reader;
pub mod watermark;

use crate::cli::ProcessArgs;
use image::DynamicImage;

#[derive(Default)]
pub struct Pipeline {
    pub resize: Option<resize::ResizeOptions>,
    pub crop: Option<crop::CropOptions>,
    pub smart_crop: Option<smart_crop::SmartCropOptions>,
    pub rotate: Option<rotate::RotateOptions>,
    pub watermark: Option<watermark::WatermarkOptions>,
    pub filter: Option<filter::FilterOptions>,
}

impl Pipeline {
    pub fn from_process_args(args: &ProcessArgs) -> anyhow::Result<Self> {
        let mut pipeline = Pipeline::default();

        // 1. Resize
        if let Some(ref resize_str) = args.resize {
            if let Some(mut opts) = resize::parse_resize_spec(resize_str) {
                opts.fit_mode = args.resize_mode;
                opts.filter = args.resize_filter;
                pipeline.resize = Some(opts);
            }
        }

        // 2. Crop
        if let Some(ref crop_str) = args.crop {
            if let Some(opts) = crop::parse_crop_spec(crop_str) {
                pipeline.crop = Some(opts);
            }
        }

        // 3. Smart Crop
        if let Some(ref smart_crop_str) = args.smart_crop {
            let mut opts = smart_crop::SmartCropOptions::default();
            if let Some((w, h)) = smart_crop::parse_aspect_ratio(smart_crop_str) {
                opts.aspect_ratio = Some((w, h));
            } else if let Some((w, h)) = smart_crop_str.split_once('x') {
                if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                    opts.target_width = Some(w);
                    opts.target_height = Some(h);
                }
            }
            pipeline.smart_crop = Some(opts);
        }

        // 4. Rotate
        if args.rotate.is_some() || args.flip_h || args.flip_v {
            pipeline.rotate = Some(rotate::RotateOptions {
                degrees: args.rotate.unwrap_or(0.0),
                flip_h: args.flip_h,
                flip_v: args.flip_v,
            });
        }

        // 5. Watermark
        if args.watermark_text.is_some() || args.watermark_image.is_some() {
            pipeline.watermark = Some(watermark::WatermarkOptions {
                text: args.watermark_text.clone(),
                image_path: args.watermark_image.clone(),
                anchor: args.watermark_anchor,
                opacity: args.watermark_opacity,
                scale: args.watermark_scale,
            });
        }

        // 6. Filters
        if args.grayscale
            || args.sepia
            || args.invert
            || args.brightness.is_some()
            || args.contrast.is_some()
            || args.blur.is_some()
            || args.sharpen.is_some()
        {
            pipeline.filter = Some(filter::FilterOptions {
                grayscale: args.grayscale,
                sepia: args.sepia,
                invert: args.invert,
                brightness: args.brightness,
                contrast: args.contrast,
                blur: args.blur,
                sharpen: args.sharpen,
                warmth: None,
                vignette: None,
                structure: None,
            });
        }

        Ok(pipeline)
    }

    /// Execute pipeline transformations in order: Crop -> Smart Crop -> Resize -> Rotate -> Filter -> Watermark
    pub fn execute(&self, img: &DynamicImage) -> anyhow::Result<DynamicImage> {
        let mut current = img.clone();

        if let Some(ref crop_opts) = self.crop {
            current = crop::crop_image(&current, crop_opts);
        }

        if let Some(ref sc_opts) = self.smart_crop {
            current = smart_crop::smart_crop(&current, sc_opts);
        }

        if let Some(ref resize_opts) = self.resize {
            current = resize::resize_image(&current, resize_opts);
        }

        if let Some(ref rotate_opts) = self.rotate {
            current = rotate::rotate_image(&current, rotate_opts);
        }

        if let Some(ref filter_opts) = self.filter {
            current = filter::apply_filters(&current, filter_opts);
        }

        if let Some(ref wm_opts) = self.watermark {
            current = watermark::apply_watermark(&current, wm_opts)?;
        }

        Ok(current)
    }
}

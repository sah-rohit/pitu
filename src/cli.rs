use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pitu",
    author = "Pitu Developers",
    version,
    about = "pitu: A fast, scriptable CLI Image Workbench for batch processing, smart entropy cropping, & CI/CD pipelines.",
    long_about = "pitu provides automated batch image manipulation including smart entropy focal-point cropping,\n\
                  resizing, format conversion, rotation, watermarking, and filtering.\n\n\
                  Examples:\n  \
                  pitu process \"photos/*.jpg\" -o ./dist --smart-crop 16:9 --watermark-text \"Pitu\" --format webp\n  \
                  pitu smart-crop input.png -o output.png --ratio 1:1\n  \
                  pitu interactive"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Global pattern or direct input path when no subcommand is provided (for drag-and-drop / simple syntax)
    #[arg(global = true)]
    pub input: Option<String>,

    /// Output directory or explicit output file path
    #[arg(short = 'o', long = "output", global = true)]
    pub output: Option<PathBuf>,

    /// Output format (e.g., png, jpg, webp, bmp, gif, tiff, ico)
    #[arg(short = 'f', long = "format", global = true)]
    pub format: Option<ImageFormatChoice>,

    /// Output quality (1-100) for lossy formats like JPEG/WebP
    #[arg(short = 'q', long = "quality", default_value_t = 85, global = true)]
    pub quality: u8,

    /// Suffix to append to output filenames (e.g., "_thumb")
    #[arg(long = "suffix", global = true)]
    pub suffix: Option<String>,

    /// Prefix to prepend to output filenames (e.g., "proc_")
    #[arg(long = "prefix", global = true)]
    pub prefix: Option<String>,

    /// Number of parallel threads to use for batch execution (default: auto)
    #[arg(short = 'j', long = "jobs", global = true)]
    pub jobs: Option<usize>,

    /// Suppress stdout messages and progress bars (CI/CD mode)
    #[arg(short = 's', long = "silent", global = true)]
    pub silent: bool,

    /// Output progress and execution status as structured JSON
    #[arg(long = "json", global = true)]
    pub json: bool,

    /// Perform a dry-run without writing output files
    #[arg(long = "dry-run", global = true)]
    pub dry_run: bool,

    /// Strip metadata and EXIF headers for privacy
    #[arg(long = "strip-exif", global = true, default_value_t = false)]
    pub strip_exif: bool,

    /// Automatically repair mismatched image extensions and header corruption
    #[arg(long = "auto-fix", global = true, default_value_t = false)]
    pub auto_fix: bool,

    /// Enhance quality, sharpness, and color pop
    #[arg(long = "enhance", global = true, default_value_t = false)]
    pub enhance: bool,

    /// Compress image to fit under maximum target file size (e.g. 500KB, 2MB)
    #[arg(long = "max-size", global = true)]
    pub max_size: Option<String>,

    /// Overwrite existing output files without prompting
    #[arg(long = "overwrite", global = true, default_value_t = true)]
    pub overwrite: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a chained multi-operation pipeline on one or multiple images/globs
    Process(ProcessArgs),

    /// Convert images between formats (PNG, JPEG, WebP, GIF, BMP, TIFF, ICO)
    Convert(ConvertArgs),

    /// Resize images with specific dimensions, aspect ratio, or bounding box
    Resize(ResizeArgs),

    /// Crop image using exact bounding box or aspect ratio
    Crop(CropArgs),

    /// Content-aware smart cropping using entropy & edge detection to preserve focal points
    SmartCrop(SmartCropArgs),

    /// Rotate or flip images
    Rotate(RotateArgs),

    /// Add text or image watermarks to images
    Watermark(WatermarkArgs),

    /// Apply visual filters (grayscale, sepia, blur, sharpen, brightness, contrast)
    Filter(FilterArgs),

    /// Execute a named preset workflow defined in pitu.toml or built-in presets
    Preset {
        /// Name of preset (e.g. web-hero, social-avatar, thumbnail-webp, watermarked-dist)
        name: String,
        /// Input file(s) or glob pattern
        input: String,
    },

    /// Render side-by-side terminal ASCII visual diff comparison (Original vs Processed)
    Diff {
        /// Input image file path
        input: String,
    },

    /// Generate starter pitu.toml configuration file in current directory
    InitConfig,

    /// Create a version-controlled snapshot commit of an image
    Sync {
        /// Target image path
        #[arg(required = true)]
        file: String,

        /// Snapshot commit message
        #[arg(short, long, default_value = "Manual snapshot commit")]
        message: String,
    },

    /// Show version snapshot history timeline for an image
    History {
        /// Target image path
        #[arg(required = true)]
        file: String,
    },

    /// Launch interactive rebase wizard to selectively toggle/remove image edit operations
    Rebase {
        /// Target image path
        #[arg(required = true)]
        file: String,
    },

    /// Revert target image to a specific version snapshot hash or index
    Revert {
        /// Target image path
        #[arg(required = true)]
        file: String,

        /// Commit hash or numeric history index to revert to
        #[arg(required = true)]
        commit: String,
    },

    /// Launch interactive step-by-step wizard dashboard
    Interactive,

    /// Show detailed program information, format support, and algorithm details
    Info,

    /// Open built-in interactive user manual and documentation
    Manual,

    /// Inspect image metadata, resolution, aspect ratio, and color specs
    Inspect {
        /// Input image file path
        #[arg(required = true)]
        file: String,
    },

    /// Render terminal ASCII preview and smart entropy focal-point heatmap
    Preview {
        /// Input image file path
        #[arg(required = true)]
        file: String,

        /// Display entropy focal-point energy heatmap instead of color thumbnail
        #[arg(long = "heatmap")]
        heatmap: bool,
    },

    /// Install global 'pitu' launcher executable in ~/.local/bin
    InstallLauncher,

    /// Generate shell completion scripts (bash, zsh, fish, powershell)
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Args, Debug, Clone, Default)]
pub struct ProcessArgs {
    /// Input file, directory, or glob pattern (e.g., "images/*.png", "**/photo.jpg")
    #[arg(required = true)]
    pub input: String,

    /// Resize target e.g. "800x600", "50%", "800x-", "-x600"
    #[arg(long = "resize")]
    pub resize: Option<String>,

    /// Resampling filter mode
    #[arg(long = "resize-filter", value_enum, default_value_t = FilterMode::Lanczos3)]
    pub resize_filter: FilterMode,

    /// Fit within bounding box without stretching ("fit", "fill", "exact", "stretch")
    #[arg(long = "resize-mode", value_enum, default_value_t = ResizeFitMode::Fit)]
    pub resize_mode: ResizeFitMode,

    /// Manual crop in format "x,y,w,h" (e.g., "10,20,400,300")
    #[arg(long = "crop")]
    pub crop: Option<String>,

    /// Smart entropy crop ratio or dimensions (e.g. "16:9", "1:1", "4:3", "800x600")
    #[arg(long = "smart-crop")]
    pub smart_crop: Option<String>,

    /// Rotation angle (90, 180, 270, or arbitrary degrees)
    #[arg(long = "rotate")]
    pub rotate: Option<f32>,

    /// Flip horizontally
    #[arg(long = "flip-h", default_value_t = false)]
    pub flip_h: bool,

    /// Flip vertically
    #[arg(long = "flip-v", default_value_t = false)]
    pub flip_v: bool,

    /// Text watermark string
    #[arg(long = "watermark-text")]
    pub watermark_text: Option<String>,

    /// Image watermark file path
    #[arg(long = "watermark-image")]
    pub watermark_image: Option<PathBuf>,

    /// Watermark anchor position
    #[arg(long = "watermark-anchor", value_enum, default_value_t = AnchorPosition::BottomRight)]
    pub watermark_anchor: AnchorPosition,

    /// Watermark opacity (0.0 to 1.0)
    #[arg(long = "watermark-opacity", default_value_t = 0.8)]
    pub watermark_opacity: f32,

    /// Watermark scale relative to base image (0.05 to 1.0)
    #[arg(long = "watermark-scale", default_value_t = 0.2)]
    pub watermark_scale: f32,

    /// Convert image to grayscale
    #[arg(long = "grayscale", default_value_t = false)]
    pub grayscale: bool,

    /// Apply sepia filter
    #[arg(long = "sepia", default_value_t = false)]
    pub sepia: bool,

    /// Invert colors
    #[arg(long = "invert", default_value_t = false)]
    pub invert: bool,

    /// Adjust brightness (-100 to 100)
    #[arg(long = "brightness")]
    pub brightness: Option<i32>,

    /// Adjust contrast (-100.0 to 100.0)
    #[arg(long = "contrast")]
    pub contrast: Option<f32>,

    /// Apply Gaussian blur radius (sigma > 0.0)
    #[arg(long = "blur")]
    pub blur: Option<f32>,

    /// Apply sharpen filter
    #[arg(long = "sharpen")]
    pub sharpen: Option<f32>,

    /// Apply color warmth adjustment (-1.0 to 1.0)
    #[arg(long = "warmth")]
    pub warmth: Option<f32>,

    /// Apply vignette strength (0.0 to 1.5)
    #[arg(long = "vignette")]
    pub vignette: Option<f32>,

    /// Apply structure / micro-contrast (0.0 to 3.0)
    #[arg(long = "structure")]
    pub structure: Option<f32>,

    /// Apply HDR Scape filter
    #[arg(long = "hdr-scape", default_value_t = false)]
    pub hdr_scape: bool,

    /// Apply Glamour Glow filter
    #[arg(long = "glamour-glow", default_value_t = false)]
    pub glamour_glow: bool,

    /// Apply Haze Removal filter
    #[arg(long = "haze-removal", default_value_t = false)]
    pub haze_removal: bool,

    /// Add a solid border frame width in pixels
    #[arg(long = "frame-width")]
    pub frame_width: Option<u32>,

    /// Adjust exposure (-5.0 to 5.0)
    #[arg(long = "exposure")]
    pub exposure: Option<f32>,

    /// Adjust saturation (0.0 to 3.0)
    #[arg(long = "saturation")]
    pub saturation: Option<f32>,

    /// Adjust shadows (-1.0 to 1.0)
    #[arg(long = "shadows")]
    pub shadows: Option<f32>,

    /// Adjust highlights (-1.0 to 1.0)
    #[arg(long = "highlights")]
    pub highlights: Option<f32>,

    /// Apply high-contrast black & white Noir filter
    #[arg(long = "noir", default_value_t = false)]
    pub noir: bool,

    /// Apply Vintage filter
    #[arg(long = "vintage", default_value_t = false)]
    pub vintage: bool,

    /// Apply Grunge texture filter
    #[arg(long = "grunge", default_value_t = false)]
    pub grunge: bool,

    /// Apply Lens Blur radius (sigma > 0.0)
    #[arg(long = "lens-blur")]
    pub lens_blur: Option<f32>,
}

#[derive(Args, Debug)]
pub struct ConvertArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Target format
    #[arg(short = 't', long = "to", value_enum, required = true)]
    pub to: ImageFormatChoice,
}

#[derive(Args, Debug)]
pub struct ResizeArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Width in pixels (optional if height provided)
    #[arg(short = 'w', long = "width")]
    pub width: Option<u32>,

    /// Height in pixels (optional if width provided)
    #[arg(short = 'H', long = "height")]
    pub height: Option<u32>,

    /// Percentage scale factor (e.g. 50.0 for 50%)
    #[arg(short = 'p', long = "percent")]
    pub percent: Option<f32>,

    /// Fit mode ("fit", "fill", "exact", "stretch")
    #[arg(long = "mode", value_enum, default_value_t = ResizeFitMode::Fit)]
    pub mode: ResizeFitMode,

    /// Filter algorithm
    #[arg(long = "filter", value_enum, default_value_t = FilterMode::Lanczos3)]
    pub filter: FilterMode,
}

#[derive(Args, Debug)]
pub struct CropArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// X coordinate of top-left corner
    #[arg(short = 'x', default_value_t = 0)]
    pub x: u32,

    /// Y coordinate of top-left corner
    #[arg(short = 'y', default_value_t = 0)]
    pub y: u32,

    /// Width of crop rectangle
    #[arg(short = 'w', long = "width")]
    pub width: Option<u32>,

    /// Height of crop rectangle
    #[arg(short = 'H', long = "height")]
    pub height: Option<u32>,

    /// Aspect ratio (e.g., "16:9", "1:1", "4:3") centered
    #[arg(short = 'r', long = "ratio")]
    pub ratio: Option<String>,
}

#[derive(Args, Debug)]
pub struct SmartCropArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Target aspect ratio (e.g. "16:9", "1:1", "4:3", "9:16") or target size "800x600"
    #[arg(short = 'r', long = "ratio")]
    pub ratio: Option<String>,

    /// Target width in pixels
    #[arg(short = 'w', long = "width")]
    pub width: Option<u32>,

    /// Target height in pixels
    #[arg(short = 'H', long = "height")]
    pub height: Option<u32>,

    /// Weight factor for entropy vs edge detection (0.0 to 1.0, default: 0.5)
    #[arg(long = "entropy-weight", default_value_t = 0.5)]
    pub entropy_weight: f32,
}

#[derive(Args, Debug)]
pub struct RotateArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Degrees to rotate clockwise (90, 180, 270, or arbitrary float)
    #[arg(short = 'd', long = "degrees", default_value_t = 90.0)]
    pub degrees: f32,

    /// Flip horizontally
    #[arg(long = "flip-h")]
    pub flip_h: bool,

    /// Flip vertically
    #[arg(long = "flip-v")]
    pub flip_v: bool,
}

#[derive(Args, Debug)]
pub struct WatermarkArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Watermark text string
    #[arg(short = 't', long = "text")]
    pub text: Option<String>,

    /// Watermark image file path
    #[arg(short = 'i', long = "image")]
    pub image: Option<PathBuf>,

    /// Anchor alignment position
    #[arg(short = 'a', long = "anchor", value_enum, default_value_t = AnchorPosition::BottomRight)]
    pub anchor: AnchorPosition,

    /// Opacity from 0.0 (transparent) to 1.0 (opaque)
    #[arg(short = 'c', long = "opacity", default_value_t = 0.8)]
    pub opacity: f32,

    /// Scale factor relative to base image width/height (0.01 to 1.0)
    #[arg(short = 's', long = "scale", default_value_t = 0.25)]
    pub scale: f32,
}

#[derive(Args, Debug)]
pub struct FilterArgs {
    /// Input file(s) or glob pattern
    #[arg(required = true)]
    pub input: Vec<String>,

    /// Grayscale filter
    #[arg(long)]
    pub grayscale: bool,

    /// Sepia tone filter
    #[arg(long)]
    pub sepia: bool,

    /// Invert colors filter
    #[arg(long)]
    pub invert: bool,

    /// Adjust brightness (-100 to 100)
    #[arg(long)]
    pub brightness: Option<i32>,

    /// Adjust contrast (-100.0 to 100.0)
    #[arg(long)]
    pub contrast: Option<f32>,

    /// Gaussian blur radius sigma
    #[arg(long)]
    pub blur: Option<f32>,

    /// Sharpen amount
    #[arg(long)]
    pub sharpen: Option<f32>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImageFormatChoice {
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
    Ico,
}

impl ImageFormatChoice {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
        }
    }

    pub fn to_image_format(&self) -> image::ImageFormat {
        match self {
            Self::Png => image::ImageFormat::Png,
            Self::Jpeg => image::ImageFormat::Jpeg,
            Self::Webp => image::ImageFormat::WebP,
            Self::Gif => image::ImageFormat::Gif,
            Self::Bmp => image::ImageFormat::Bmp,
            Self::Tiff => image::ImageFormat::Tiff,
            Self::Ico => image::ImageFormat::Ico,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    #[default]
    Lanczos3,
}

impl FilterMode {
    pub fn to_filter_type(&self) -> image::imageops::FilterType {
        match self {
            Self::Nearest => image::imageops::FilterType::Nearest,
            Self::Triangle => image::imageops::FilterType::Triangle,
            Self::CatmullRom => image::imageops::FilterType::CatmullRom,
            Self::Gaussian => image::imageops::FilterType::Gaussian,
            Self::Lanczos3 => image::imageops::FilterType::Lanczos3,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResizeFitMode {
    #[default]
    Fit,
    Fill,
    Exact,
    Stretch,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AnchorPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

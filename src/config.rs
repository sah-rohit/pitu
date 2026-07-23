use crate::cli::ProcessArgs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PituConfig {
    #[serde(default)]
    pub presets: BTreeMap<String, PresetConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PresetConfig {
    pub description: Option<String>,
    pub resize: Option<String>,
    pub smart_crop: Option<String>,
    pub crop: Option<String>,
    pub rotate: Option<f32>,
    pub watermark_text: Option<String>,
    pub watermark_image: Option<String>,
    pub format: Option<String>,
    pub quality: Option<u8>,
    pub grayscale: Option<bool>,
    pub sepia: Option<bool>,
}

impl PresetConfig {
    pub fn to_process_args(&self, input: String) -> ProcessArgs {
        let mut args = ProcessArgs {
            input,
            resize: self.resize.clone(),
            smart_crop: self.smart_crop.clone(),
            crop: self.crop.clone(),
            rotate: self.rotate,
            watermark_text: self.watermark_text.clone(),
            grayscale: self.grayscale.unwrap_or(false),
            sepia: self.sepia.unwrap_or(false),
            ..Default::default()
        };

        if let Some(ref img_path) = self.watermark_image {
            args.watermark_image = Some(std::path::PathBuf::from(img_path));
        }

        args
    }
}

pub fn load_config() -> PituConfig {
    let local_path = Path::new("pitu.toml");
    if local_path.exists() {
        if let Ok(content) = fs::read_to_string(local_path) {
            if let Ok(config) = toml::from_str::<PituConfig>(&content) {
                return config;
            }
        }
    }

    get_builtin_config()
}

pub fn get_builtin_config() -> PituConfig {
    let mut presets = BTreeMap::new();

    presets.insert(
        "web-hero".to_string(),
        PresetConfig {
            description: Some("16:9 Smart Crop resized to 1920x1080 WebP".into()),
            smart_crop: Some("16:9".into()),
            resize: Some("1920x1080".into()),
            format: Some("webp".into()),
            quality: Some(85),
            ..Default::default()
        },
    );

    presets.insert(
        "social-avatar".to_string(),
        PresetConfig {
            description: Some("1:1 Square Smart Crop resized to 500x500 WebP".into()),
            smart_crop: Some("1:1".into()),
            resize: Some("500x500".into()),
            format: Some("webp".into()),
            quality: Some(85),
            ..Default::default()
        },
    );

    presets.insert(
        "thumbnail-webp".to_string(),
        PresetConfig {
            description: Some("16:9 Smart Crop thumbnail resized to 400x225 WebP".into()),
            smart_crop: Some("16:9".into()),
            resize: Some("400x225".into()),
            format: Some("webp".into()),
            quality: Some(80),
            ..Default::default()
        },
    );

    presets.insert(
        "watermarked-dist".to_string(),
        PresetConfig {
            description: Some("Overlays default watermark text on images".into()),
            watermark_text: Some("© PITU WORKBENCH".into()),
            format: Some("webp".into()),
            ..Default::default()
        },
    );

    PituConfig { presets }
}

pub fn create_default_config_file() -> anyhow::Result<()> {
    let path = Path::new("pitu.toml");
    if path.exists() {
        println!("  pitu.toml already exists in current directory.");
        return Ok(());
    }

    let default_content = r#"# pitu.toml - Pitu Image Workbench Presets

[presets.web-hero]
description = "16:9 Widescreen Smart Crop scaled to 1920x1080 WebP"
smart_crop = "16:9"
resize = "1920x1080"
format = "webp"
quality = 85

[presets.social-avatar]
description = "1:1 Square Focal-Point Crop scaled to 500x500 WebP"
smart_crop = "1:1"
resize = "500x500"
format = "webp"
quality = 85

[presets.thumbnail-webp]
description = "400x225 WebP Thumbnail"
smart_crop = "16:9"
resize = "400x225"
format = "webp"
quality = 80

[presets.watermarked-dist]
description = "Add copyright watermark text to images"
watermark_text = "© PITU WORKBENCH"
format = "webp"
"#;

    fs::write(path, default_content)?;
    println!("  ✔ Created starter 'pitu.toml' in current directory!");
    Ok(())
}

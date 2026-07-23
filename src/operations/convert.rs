use crate::cli::ImageFormatChoice;
use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageFormat};
use std::path::Path;

pub fn convert_format_to_bytes(
    img: &DynamicImage,
    target_format: ImageFormatChoice,
    quality: u8,
) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    match target_format {
        ImageFormatChoice::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder.encode_image(img)?;
        }
        ImageFormatChoice::Webp => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::WebP)?;
        }
        ImageFormatChoice::Png => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)?;
        }
        ImageFormatChoice::Gif => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Gif)?;
        }
        ImageFormatChoice::Bmp => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Bmp)?;
        }
        ImageFormatChoice::Tiff => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Tiff)?;
        }
        ImageFormatChoice::Ico => {
            img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Ico)?;
        }
    }
    Ok(buffer)
}

pub fn convert_format(
    img: &DynamicImage,
    target_path: &Path,
    target_format: ImageFormatChoice,
    quality: u8,
) -> anyhow::Result<()> {
    if let Some(parent) = target_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let bytes = convert_format_to_bytes(img, target_format, quality)?;
    std::fs::write(target_path, bytes)?;
    Ok(())
}

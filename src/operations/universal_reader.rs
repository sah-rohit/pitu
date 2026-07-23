use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::path::Path;

pub struct UniversalReadResult {
    pub image: DynamicImage,
    pub detected_format: String,
    pub extension_mismatch: bool,
    pub expected_ext: String,
    pub health_score: u8,
    pub polyglot_extracted: bool,
    pub base64_decoded: bool,
}

/// Next-Gen Universal Image Reader with Polyglot Byte Scanner, Base64 Decoder & Health Scoring
pub fn load_universal_image(path: &Path) -> anyhow::Result<UniversalReadResult> {
    let raw_bytes = fs::read(path)?;
    if raw_bytes.is_empty() {
        return Err(anyhow::anyhow!("File is empty (0 bytes)"));
    }

    let mut polyglot_extracted = false;
    let mut base64_decoded = false;

    // 1. Check for Base64 Data URI strings
    let bytes = if let Some(decoded) = try_decode_base64_data_uri(&raw_bytes) {
        base64_decoded = true;
        decoded
    } else if let Some(poly_stream) = extract_polyglot_image_stream(&raw_bytes) {
        polyglot_extracted = true;
        poly_stream
    } else {
        raw_bytes
    };

    // Check if non-image HTML text file without valid embedded image stream
    if is_html_or_text_doc(&bytes) {
        return Err(anyhow::anyhow!(
            "File is an HTML webpage or text document (e.g. 404/Error page), not a binary image"
        ));
    }

    let actual_ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mut health_score = 100u8;

    // 2. Try magic-byte format guessing
    let (img, detected_fmt_enum) = match image::guess_format(&bytes) {
        Ok(fmt) => match image::load_from_memory_with_format(&bytes, fmt) {
            Ok(loaded) => (loaded, Some(fmt)),
            Err(_) => {
                health_score = health_score.saturating_sub(25);
                try_fallback_decoders(&bytes)?
            }
        },
        Err(_) => {
            health_score = health_score.saturating_sub(30);
            try_fallback_decoders(&bytes)?
        }
    };

    let detected_fmt_str = match detected_fmt_enum {
        Some(ImageFormat::Jpeg) => "JPEG",
        Some(ImageFormat::Png) => "PNG",
        Some(ImageFormat::WebP) => "WebP",
        Some(ImageFormat::Gif) => "GIF",
        Some(ImageFormat::Bmp) => "BMP",
        Some(ImageFormat::Tiff) => "TIFF",
        Some(ImageFormat::Ico) => "ICO",
        Some(ImageFormat::Tga) => "TGA",
        Some(ImageFormat::Pnm) => "PNM",
        Some(ImageFormat::Hdr) => "HDR",
        Some(ImageFormat::Farbfeld) => "Farbfeld",
        Some(ImageFormat::Dds) => "DDS",
        Some(ImageFormat::Qoi) => "QOI",
        _ => "IMAGE",
    };

    let expected_ext = match detected_fmt_enum {
        Some(ImageFormat::Jpeg) => "jpg",
        Some(ImageFormat::Png) => "png",
        Some(ImageFormat::WebP) => "webp",
        Some(ImageFormat::Gif) => "gif",
        Some(ImageFormat::Bmp) => "bmp",
        Some(ImageFormat::Tiff) => "tiff",
        Some(ImageFormat::Ico) => "ico",
        Some(ImageFormat::Tga) => "tga",
        Some(ImageFormat::Pnm) => "pnm",
        Some(ImageFormat::Hdr) => "hdr",
        Some(ImageFormat::Farbfeld) => "ff",
        Some(ImageFormat::Dds) => "dds",
        Some(ImageFormat::Qoi) => "qoi",
        _ => &actual_ext,
    };

    let extension_mismatch = !actual_ext.is_empty()
        && expected_ext != actual_ext
        && !(actual_ext == "jpeg" && expected_ext == "jpg")
        && !(actual_ext == "jpg" && expected_ext == "jpeg");

    if extension_mismatch {
        health_score = health_score.saturating_sub(15);
    }
    if polyglot_extracted || base64_decoded {
        health_score = health_score.saturating_sub(10);
    }

    Ok(UniversalReadResult {
        image: img,
        detected_format: detected_fmt_str.to_string(),
        extension_mismatch,
        expected_ext: expected_ext.to_string(),
        health_score,
        polyglot_extracted,
        base64_decoded,
    })
}

fn try_decode_base64_data_uri(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 20 {
        return None;
    }
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_lowercase();
    if let Some(idx) = text.find(";base64,") {
        let payload_start = idx + 8;
        if payload_start < bytes.len() {
            let base64_str = String::from_utf8_lossy(&bytes[payload_start..])
                .trim()
                .to_string();
            if let Ok(decoded) = BASE64.decode(base64_str.as_bytes()) {
                return Some(decoded);
            }
        }
    }
    None
}

/// Deep Polyglot Scanner: searches first 2048 bytes for embedded image signatures
fn extract_polyglot_image_stream(bytes: &[u8]) -> Option<Vec<u8>> {
    let scan_limit = bytes.len().min(2048);
    let sample = &bytes[..scan_limit];

    // JPEG Magic: FF D8 FF
    if let Some(pos) = find_subslice(sample, &[0xFF, 0xD8, 0xFF]) {
        if pos > 0 {
            return Some(bytes[pos..].to_vec());
        }
    }

    // PNG Magic: 89 50 4E 47
    if let Some(pos) = find_subslice(sample, &[0x89, 0x50, 0x4E, 0x47]) {
        if pos > 0 {
            return Some(bytes[pos..].to_vec());
        }
    }

    // GIF Magic: GIF8
    if let Some(pos) = find_subslice(sample, b"GIF8") {
        if pos > 0 {
            return Some(bytes[pos..].to_vec());
        }
    }

    // WebP Magic: RIFF ... WEBP
    if let Some(pos) = find_subslice(sample, b"RIFF") {
        if pos + 8 < bytes.len() && &bytes[pos + 8..pos + 12] == b"WEBP" {
            if pos > 0 {
                return Some(bytes[pos..].to_vec());
            }
        }
    }

    None
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn is_html_or_text_doc(bytes: &[u8]) -> bool {
    let sample_size = bytes.len().min(512);
    let sample = &bytes[..sample_size];

    let text_sample = String::from_utf8_lossy(sample).to_lowercase();
    (text_sample.contains("<!doctype html")
        || text_sample.contains("<html")
        || text_sample.contains("404 not found")
        || text_sample.contains("<head>"))
        && !text_sample.contains(";base64,")
}

fn try_fallback_decoders(bytes: &[u8]) -> anyhow::Result<(DynamicImage, Option<ImageFormat>)> {
    if let Ok(loaded) = image::load_from_memory(bytes) {
        return Ok((loaded, None));
    }

    let formats = vec![
        ImageFormat::Jpeg,
        ImageFormat::Png,
        ImageFormat::WebP,
        ImageFormat::Gif,
        ImageFormat::Bmp,
        ImageFormat::Tiff,
        ImageFormat::Ico,
        ImageFormat::Tga,
        ImageFormat::Pnm,
        ImageFormat::Hdr,
        ImageFormat::Farbfeld,
        ImageFormat::Dds,
        ImageFormat::Qoi,
    ];

    for fmt in formats {
        if let Ok(loaded) = image::load_from_memory_with_format(bytes, fmt) {
            return Ok((loaded, Some(fmt)));
        }
    }

    Err(anyhow::anyhow!(
        "Failed to decode image: unsupported format or severely corrupted binary image stream"
    ))
}

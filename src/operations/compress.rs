use crate::cli::ImageFormatChoice;
use crate::operations::convert::convert_format_to_bytes;
use image::DynamicImage;

/// Parse human readable file size specs like "500KB", "2MB", "1.5MB", "800B"
pub fn parse_size_bytes(spec: &str) -> Option<u64> {
    let s = spec.trim().to_uppercase();
    if s.is_empty() {
        return None;
    }

    if let Some(num_str) = s.strip_suffix("MB") {
        let val: f64 = num_str.trim().parse().ok()?;
        Some((val * 1_024.0 * 1_024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix("KB") {
        let val: f64 = num_str.trim().parse().ok()?;
        Some((val * 1_024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix("B") {
        let val: u64 = num_str.trim().parse().ok()?;
        Some(val)
    } else if let Ok(val) = s.parse::<u64>() {
        Some(val)
    } else {
        None
    }
}

/// Binary search quality compressor: finds maximum quality yielding file size <= target_bytes
pub fn compress_to_max_size(
    img: &DynamicImage,
    target_bytes: u64,
    format: ImageFormatChoice,
) -> anyhow::Result<(Vec<u8>, u32)> {
    let mut low = 5u32;
    let mut high = 100u32;
    let mut best_bytes = Vec::new();
    let mut best_quality = 85u32;

    // Check if initial encode at 85% is already under target limit
    if let Ok(bytes) = convert_format_to_bytes(img, format, 85) {
        if (bytes.len() as u64) <= target_bytes {
            return Ok((bytes, 85));
        }
    }

    while low <= high {
        let mid = (low + high) / 2;
        match convert_format_to_bytes(img, format, mid as u8) {
            Ok(bytes) => {
                let size = bytes.len() as u64;
                if size <= target_bytes {
                    best_bytes = bytes;
                    best_quality = mid;
                    low = mid + 1; // Try higher quality
                } else {
                    if mid == 0 {
                        break;
                    }
                    high = mid - 1; // Try lower quality
                }
            }
            Err(_) => {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            }
        }
    }

    if best_bytes.is_empty() {
        // Fallback: encode at lowest quality 5%
        let fallback = convert_format_to_bytes(img, format, 5)?;
        Ok((fallback, 5))
    } else {
        Ok((best_bytes, best_quality))
    }
}

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/utils.rs"]
mod utils;
#[path = "../src/batch.rs"]
mod batch;
#[path = "../src/config.rs"]
mod config;
#[path = "../src/operations/mod.rs"]
mod operations;
#[path = "../src/ui/mod.rs"]
mod ui;
#[path = "../src/manual.rs"]
mod manual;
#[path = "../src/interactive.rs"]
mod interactive;
#[path = "../src/session.rs"]
mod session;
#[path = "../src/versioning.rs"]
mod versioning;
#[path = "../src/operations/universal_reader.rs"]
mod universal_reader;
#[path = "../src/operations/auto_fix.rs"]
mod auto_fix;

use image::RgbImage;
use std::fs::File;
use std::io::BufWriter;
use universal_reader::load_universal_image;

#[test]
fn test_universal_reader_mismatched_extension() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fake_png_path = temp_dir.path().join("fake_image.png");

    // Save a JPEG image with a .png extension
    let img_buf = RgbImage::new(100, 100);
    let file = File::create(&fake_png_path).unwrap();
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 85);
    encoder.encode_image(&img_buf).unwrap();
    drop(writer);

    let res = load_universal_image(&fake_png_path).unwrap();
    assert_eq!(res.detected_format, "JPEG");
    assert!(res.extension_mismatch);
    assert_eq!(res.expected_ext, "jpg");
}

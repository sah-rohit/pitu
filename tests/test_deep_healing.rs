#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/utils.rs"]
mod utils;
#[path = "../src/interactive.rs"]
mod interactive;
#[path = "../src/ui/mod.rs"]
mod ui;
#[path = "../src/operations/mod.rs"]
mod operations;
#[path = "../src/config.rs"]
mod config;
#[path = "../src/manual.rs"]
mod manual;
#[path = "../src/batch.rs"]
mod batch;

#[path = "../src/session.rs"]
mod session;
#[path = "../src/versioning.rs"]
mod versioning;

use operations::auto_fix::auto_fix_and_repair;
use operations::universal_reader::load_universal_image;
use std::fs::File;
use std::io::Write;

#[test]
fn test_base64_data_uri_rescue() {
    let temp_dir = tempfile::tempdir().unwrap();
    let base64_file = temp_dir.path().join("data_uri.txt");

    // Write a Base64 encoded 1x1 PNG data URI string
    let b64_str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let mut f = File::create(&base64_file).unwrap();
    f.write_all(b64_str.as_bytes()).unwrap();

    let res = load_universal_image(&base64_file).unwrap();
    assert_eq!(res.detected_format, "PNG");
    assert!(res.base64_decoded);

    let fix_res = auto_fix_and_repair(&base64_file).unwrap();
    assert_eq!(fix_res.image.width(), 1);
    assert_eq!(fix_res.image.height(), 1);
}

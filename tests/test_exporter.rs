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

use cli::ImageFormatChoice;
use std::path::{Path, PathBuf};
use ui::exporter::{compute_target_path, NamingStrategy, SaveOptions};

#[test]
fn test_compute_target_path_save_as_copy() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("photo.jpg");

    let save_opts = SaveOptions {
        destination_dir: temp_dir.path().to_path_buf(),
        naming_strategy: NamingStrategy::SaveAsCopy,
        format: Some(ImageFormatChoice::Webp),
    };

    let target = compute_target_path(&input_path, &save_opts);
    assert_eq!(target.file_name().unwrap(), "photo_copy.webp");
}

#[test]
fn test_compute_target_path_custom_name() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("photo.jpg");

    let save_opts = SaveOptions {
        destination_dir: temp_dir.path().to_path_buf(),
        naming_strategy: NamingStrategy::CustomName("my_hero".to_string()),
        format: Some(ImageFormatChoice::Png),
    };

    let target = compute_target_path(&input_path, &save_opts);
    assert_eq!(target.file_name().unwrap(), "my_hero.png");
}

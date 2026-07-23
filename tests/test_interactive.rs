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
#[path = "../src/session.rs"]
mod session;
#[path = "../src/versioning.rs"]
mod versioning;
#[path = "../src/interactive.rs"]
mod interactive;

use utils::sanitize_input_path;

#[test]
fn test_path_sanitization() {
    // 1. Double quotes
    assert_eq!(
        sanitize_input_path("\"/home/user/Pictures/photo.jpg\""),
        "/home/user/Pictures/photo.jpg"
    );

    // 2. Single quotes
    assert_eq!(
        sanitize_input_path("'/home/user/Pictures/photo.jpg'"),
        "/home/user/Pictures/photo.jpg"
    );

    // 3. File URL protocol (file:///)
    assert_eq!(
        sanitize_input_path("file:///home/user/Pictures/photo.jpg"),
        "/home/user/Pictures/photo.jpg"
    );

    // 4. Escaped spaces (\ )
    assert_eq!(
        sanitize_input_path("/home/user/My\\ Photos/cat\\ picture.png"),
        "/home/user/My Photos/cat picture.png"
    );
}

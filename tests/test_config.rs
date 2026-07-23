#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/config.rs"]
mod config;

use config::{get_builtin_config, PresetConfig};

#[test]
fn test_builtin_presets() {
    let cfg = get_builtin_config();
    assert!(cfg.presets.contains_key("web-hero"));
    assert!(cfg.presets.contains_key("social-avatar"));
    assert!(cfg.presets.contains_key("thumbnail-webp"));

    let web_hero = cfg.presets.get("web-hero").unwrap();
    assert_eq!(web_hero.smart_crop.as_deref(), Some("16:9"));
    assert_eq!(web_hero.resize.as_deref(), Some("1920x1080"));

    let proc_args = web_hero.to_process_args("input.png".to_string());
    assert_eq!(proc_args.input, "input.png");
    assert_eq!(proc_args.smart_crop.as_deref(), Some("16:9"));
}

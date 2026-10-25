use super::*;

/// The whole point of the folder move: the definition travels with the repo, so it is read
/// from the config root and nowhere machine-local.
#[test]
fn the_adapters_folder_is_inside_the_config_repo() {
    let cfg = shall::config::Config {
        config_root: std::path::PathBuf::from(if cfg!(windows) { r"C:\repo" } else { "/repo" }),
        ..Default::default()
    };
    let layout = cfg.layout();
    for f in [
        layout.adapter_backends_file(),
        layout.adapter_settings_file(),
        layout.adapter_bootstrap_file(),
    ] {
        assert!(f.starts_with(cfg.config_root()), "{:?} escaped the repo", f);
        assert!(f.parent().unwrap().ends_with("adapters"), "{:?}", f);
    }
}

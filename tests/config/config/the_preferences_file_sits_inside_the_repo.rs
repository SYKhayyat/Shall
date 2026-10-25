use super::*;

#[test]
fn the_preferences_file_sits_inside_the_repo() {
    let sandbox = std::path::Path::new(if cfg!(windows) {
        r"C:\shall-test-sandbox"
    } else {
        "/tmp/shall-test-sandbox"
    });
    let cfg = Config::sandboxed(sandbox);
    assert_eq!(cfg.preferences_file, cfg.layout().preferences_file());
}

use super::*;

/// A row that cannot both start and stop is refused rather than half-loaded (U36).
#[test]
fn a_row_missing_start_or_stop_is_refused() {
    let toml = r#"
[[init]]
name = "broken"
detect = "brokenctl"
start = [["brokenctl", "up", "{name}"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    assert!(!providers(file.init).iter().any(|p| p.name == "broken"));
}

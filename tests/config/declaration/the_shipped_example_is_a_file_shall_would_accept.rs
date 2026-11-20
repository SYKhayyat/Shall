use super::*;

#[test]
fn the_shipped_example_is_a_file_shall_would_accept() {
    // It documented eight keys that had been deleted from this struct. Every one was
    // silently ignored, so the example read as a working config and was not one.
    let example = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/preferences.toml");
    let text = std::fs::read_to_string(example).expect("examples/preferences.toml");
    if let Err(e) = toml::from_str::<Config>(&text) {
        panic!("examples/preferences.toml documents a key that no longer exists: {e}");
    }
}

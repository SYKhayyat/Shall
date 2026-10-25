use super::*;

#[test]
fn the_shipped_example_config_calls_functions_that_exist() {
    // The bug reached a user through `examples/preferences.toml`, which documented
    // `exec(...)`. A doc that names a function nothing registers is the same defect one
    // layer out, so the example is pinned to the shell's real name.
    let example = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/preferences.toml"),
    )
    .expect("examples/preferences.toml");
    assert!(
        !example.contains("exec("),
        "the example config calls `exec(`, which no engine registers; the shell is `sh(`"
    );
    assert!(
        example.contains("sh(\"systemctl enable docker\")"),
        "the example's Rhai hook should demonstrate the real shell function"
    );
}

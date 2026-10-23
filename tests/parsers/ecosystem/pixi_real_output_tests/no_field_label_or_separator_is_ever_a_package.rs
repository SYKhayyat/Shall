use super::*;

/// What the old parser actually produced, asserted so it cannot come back: field labels,
/// separators and bare version numbers offered to a user as packages to install.
#[test]
fn no_field_label_or_separator_is_ever_a_package() {
    let names: Vec<String> = pixi_search(SEARCH, "pixi")
        .into_iter()
        .map(|p| p.name)
        .collect();
    for junk in [
        "-",
        "...",
        "Name",
        "Version",
        "Build",
        "Size",
        "License",
        "Timestamp",
        "Subdir",
        "URL",
        "MD5",
        "SHA256",
        "Dependencies:",
        "Using",
        "15.1.0",
        "15.0.0",
    ] {
        assert!(
            !names.iter().any(|n| n == junk),
            "`{junk}` came back: {names:?}"
        );
    }
}

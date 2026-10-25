use super::*;

/// The case the hand-rolled parsers were exposed to. A coloured listing parsed raw yields
/// package names with escape bytes welded to them — names that match nothing the installed
/// listing reports, which is permanent phantom drift rather than a visible failure.
#[test]
fn a_coloured_listing_yields_clean_names() {
    let listing = "\u{1b}[1mripgrep\u{1b}[0m 14.1.0\r\n\u{1b}[1mfd\u{1b}[0m 10.2.0\r\n";
    let clean = sanitize(listing);
    let names: Vec<&str> = clean
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert_eq!(names, ["ripgrep", "fd"]);

    // And the same listing parsed raw, which is what six backends did: the escape bytes
    // ride along on the name.
    let raw: Vec<&str> = listing
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert_ne!(raw, ["ripgrep", "fd"]);
}

use super::*;

/// Legend lines and nothing else is not an empty machine. `pkgin search` for a term that
/// matches nothing prints exactly its legend, and a caller that read that as *"no packages
/// installed"* would be answering a search with the machine's state.
#[test]
fn a_listing_of_nothing_but_legend_is_reported() {
    let err = parse_pkgin("=: package is installed and up to date\n")
        .expect_err("a legend alone is not a package listing");
    assert_eq!(err.backend, "pkgin");
}

use super::*;

#[test]
fn a_plural_row_header_is_reported_rather_than_silently_dropped() {
    // The whole reason this module exists. `[[backends]]` is valid TOML describing a table
    // no reader opens: no parse error, no warning, and the `mymgr:` line fails much later
    // with a message about an unknown backend.
    assert_eq!(
        standing(
            &[("backends", "[[backends]]\nname = \"mymgr\"\n")],
            "backends"
        ),
        Standing::NoRows
    );
}

use super::*;

/// The ratchet. Rows that nobody has asked may only become fewer.
#[test]
fn the_unasked_rows_may_only_become_fewer() {
    let unasked: Vec<&str> = TERMINATORS
        .iter()
        .filter(|t| matches!(t.evidence, Evidence::Unasked(_)))
        .map(|t| t.binary)
        .collect();
    eprintln!(
        "terminator table: {} rows, {} measured, {} never asked",
        TERMINATORS.len(),
        TERMINATORS.len() - unasked.len(),
        unasked.len()
    );
    assert!(
        unasked.len() <= UNASKED_CEILING,
        "{} rows say nobody asked, and the ceiling is {}: {:?}\n\n\
         A new row on a family resemblance is how this file broke four times. Measure it \
         — `tests/core/argv/terminator_probe_tests.rs` in an image that has the tool — and lower \
         the ceiling.",
        unasked.len(),
        UNASKED_CEILING,
        unasked
    );
}

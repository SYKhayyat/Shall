use super::*;

/// The rows ship compiled in, so a machine with no `adapters/` directory still gets them —
/// which is the whole point: `mix:` has to work on a fresh clone.
#[test]
fn the_shipped_rows_are_available_without_a_repo_file() {
    let h = harness(&[]);
    let rows = h.prereqs().rows();
    for manager in ["mix", "asdf", "opam"] {
        assert!(
            rows.iter().any(|r| r.manager == manager),
            "{manager} has no shipped row"
        );
    }
}

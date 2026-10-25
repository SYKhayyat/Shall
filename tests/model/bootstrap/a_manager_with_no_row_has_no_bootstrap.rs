use super::*;

#[test]
fn a_manager_with_no_row_has_no_bootstrap() {
    let rows = vec![row("brew", None)];
    assert!(for_manager(&rows, "nix", "linux").is_none());
}

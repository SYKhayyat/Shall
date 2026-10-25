use super::*;

#[test]
fn a_path_that_climbs_out_is_refused() {
    let dir = TempDir::new().unwrap();
    let err = resolve_target(dir.path(), Some("../../.bashrc")).unwrap_err();
    assert!(err.to_string().contains("climbs out"));
}

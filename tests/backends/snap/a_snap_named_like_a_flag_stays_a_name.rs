use super::*;

#[test]
fn a_snap_named_like_a_flag_stays_a_name() {
    let args = install_args(&spec_with("--classic", &[]));
    assert_eq!(args, ["install", "--", "--classic"]);
}

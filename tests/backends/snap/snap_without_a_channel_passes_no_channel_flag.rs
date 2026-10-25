use super::*;

#[test]
fn snap_without_a_channel_passes_no_channel_flag() {
    let args = install_args(&spec_with("code", &[]));
    assert_eq!(args, ["install", "--", "code"]);
}

use super::*;

#[test]
fn snap_channel_reaches_the_command() {
    let args = install_args(&spec_with("code", &[("channel", "edge")]));
    assert_eq!(args, ["install", "--channel", "edge", "--", "code"]);
}

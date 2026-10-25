use super::*;

#[test]
fn a_flag_name_does_not_match_a_longer_one() {
    // The exact trap in helm 3's own help.
    assert!(!mentions_flag(
        "      --kube-insecure-skip-tls-verify   if true, …",
        "--verify"
    ));
    assert!(!mentions_flag("      --ca-file string   …", "--ca"));
    // And it must still find the real thing, in every shape help text writes it.
    assert!(mentions_flag("Use --verify=false to skip.", "--verify"));
    assert!(mentions_flag(
        "      --verify   verify the package",
        "--verify"
    ));
    assert!(mentions_flag("ends the line with --verify", "--verify"));
}

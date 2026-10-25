use super::*;

#[test]
fn extract_targets_does_not_adopt_a_flag_s_value() {
    // `apt-get install -t unstable curl` previously recorded `unstable` as a package.
    assert_eq!(
        extract_targets(&s(&["apt-get", "install", "-t", "unstable", "curl"])),
        vec!["curl"]
    );
    assert_eq!(
        extract_targets(&s(&[
            "apt",
            "install",
            "--target-release",
            "unstable",
            "curl"
        ])),
        vec!["curl"]
    );
}

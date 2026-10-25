use super::*;

/// A package with no pin has nothing to explain, however its install failed — and the same
/// name under a different manager is a different key, which is the sibling a `contains`
/// over the whole file would have got wrong.
#[test]
fn a_package_that_is_not_pinned_is_never_blamed_on_a_pin() {
    let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
    assert!(on_install_failure(&config, "apt", "curl", APT_SAID).is_none());

    // And the token boundary: an error quoting a DIFFERENT version that merely contains
    // the pinned one as a substring (11.23 contains 1.2) is not the lockfile's doing.
    let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"1.2"}}"#);
    assert!(
        on_install_failure(
            &config,
            "apt",
            "libudev1",
            "E: Version '11.23' was not found"
        )
        .is_none(),
        "a substring hit blamed the pin"
    );
    assert!(
        on_install_failure(&config, "apt", "libudev1", "E: Version '1.2' was not found").is_some(),
        "the exact version, bounded by spaces, still gets advice"
    );
    assert!(
        on_install_failure(&config, "cargo", "libudev1", APT_SAID).is_none(),
        "`cargo:libudev1` is not `apt:libudev1`, and only one of them is pinned"
    );
}

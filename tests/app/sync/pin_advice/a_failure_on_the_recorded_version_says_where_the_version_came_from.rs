use super::*;

/// The whole point: the failure names a version, the lockfile holds that version, so the
/// reader is told where it came from and given three ways out.
#[test]
fn a_failure_on_the_recorded_version_says_where_the_version_came_from() {
    let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
    let advice = on_install_failure(&config, "apt", "libudev1", APT_SAID)
        .expect("a pinned package that failed on its pin must be explained");

    assert!(advice.contains("locks/versions.json"), "{advice}");
    assert!(advice.contains("shall upgrade libudev1"), "{advice}");
    assert!(
        advice.contains("shall unlock versions libudev1"),
        "{advice}"
    );
    assert!(advice.contains("sync --upgrade"), "{advice}");
    assert!(
        advice.contains("[lock] replay"),
        "the permanent fix is not offered: {advice}"
    );
}

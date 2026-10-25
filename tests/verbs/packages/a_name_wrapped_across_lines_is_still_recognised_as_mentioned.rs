use super::*;

/// pixi wraps its output inside the package name. Attribution has to survive that, or a
/// line whose name is plainly in the output reads as one nobody mentioned. Captured from
/// pixi on this host, 2026-07-29.
#[test]
fn a_name_wrapped_across_lines_is_still_recognised_as_mentioned() {
    let wrapped = "  × failed to solve the environment\n  ╰─▶ Cannot solve the request \
                   because of: No candidates were found for shall-\n      \
                   no-such-pkg-zzz *.\n";
    assert!(
        !wrapped.contains("shall-no-such-pkg-zzz"),
        "the fixture no longer wraps, so it cannot test the wrap"
    );
    assert!(mentions_package(wrapped, "shall-no-such-pkg-zzz"));
    assert!(!mentions_package(wrapped, "some-other-package"));
}

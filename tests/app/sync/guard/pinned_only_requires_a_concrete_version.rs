use super::*;

/// **Which lines were refused, not how many.** This asserted `os.len() == 2` and nothing
/// else, so swapping `v != "latest"` for `v == "latest"` kept the count at two while
/// reversing the verdict on every line: `@version=latest` passed as pinned and `@version=1.6`
/// was refused as floating. A count is invariant under a permutation, which is exactly what
/// a comparison flip produces.
#[test]
fn pinned_only_requires_a_concrete_version() {
    let guard = shall::config::GuardSettings {
        pinned_only: true,
        ..Default::default()
    };
    let os = inspect_desired(
        &guard,
        &desired(&[
            ("apt", "curl", None),           // no version -> refused
            ("apt", "wget", Some("latest")), // floating -> refused
            ("apt", "tree", Some("*")),      // floating -> refused
            ("apt", "htop", Some("")),       // empty is not a version -> refused
            ("apt", "jq", Some("1.6")),      // pinned -> ok
        ]),
    );

    let mut refused: Vec<&str> = os
        .iter()
        .map(|o| match o {
            Objection::Unpinned { key } => key.as_str(),
            other => panic!("pinned_only raised something that is not Unpinned: {other:?}"),
        })
        .collect();
    refused.sort_unstable();
    assert_eq!(
        refused,
        ["apt:curl", "apt:htop", "apt:tree", "apt:wget"],
        "a concrete version is the only thing that satisfies pinned_only, and `apt:jq` is \
             the only line here that has one"
    );
}

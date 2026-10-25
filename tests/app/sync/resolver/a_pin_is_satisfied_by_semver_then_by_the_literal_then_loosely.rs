use super::*;

/// **The three tiers of `satisfies_constraint`, which decides whether a pin is already met.**
///
/// A `Lacks` verdict is what sends a package back to its manager, so this function decides
/// whether `sync` reinstalls. It tries SemVer, then literal equality, then a loose compare,
/// and the fallbacks exist because managers ship versions SemVer cannot parse — Debian
/// epochs, distro suffixes, four-part numbers. It had no test of its own: its name appeared
/// once in this file, at its only call site.
///
/// A table rather than one case each, because the interesting part is which tier answers.
#[tokio::test]
async fn a_pin_is_satisfied_by_semver_then_by_the_literal_then_loosely() {
    let r = repo(&[("modules/starter.txt", "")]);
    let resolver = StateResolver::new(&r.config, Arc::new(BackendRegistry::new()), false).await;

    for (version, constraint, want, why) in [
        // Tier 0: the wildcards mean "any version", so nothing is ever reinstalled for them.
        ("1.2.3", "latest", true, "`latest` accepts anything"),
        ("1.2.3", "*", true, "`*` accepts anything"),
        ("1.2.3", "", true, "an empty constraint accepts anything"),
        // Tier 1: real SemVer on both sides, so the range operators mean what they say.
        ("1.2.3", "^1.2", true, "^1.2 admits 1.2.3"),
        ("2.0.0", "^1.2", false, "^1.2 must not admit 2.0.0"),
        ("1.2.3", ">1.0.0", true, "a satisfied > range"),
        ("0.9.0", ">1.0.0", false, "an unsatisfied > range"),
        ("1.2.3", "1.2.3", true, "an exact SemVer version"),
        // Tier 2: not SemVer on either side, so only the literal can answer.
        (
            "1:2.3-4ubuntu1",
            "1:2.3-4ubuntu1",
            true,
            "a Debian epoch matches itself literally",
        ),
        (
            "1:2.3-4ubuntu1",
            "1:2.3-4ubuntu2",
            false,
            "two different distro suffixes are not the same version",
        ),
        // Tier 3: neither SemVer nor literal, so the loose comparison decides. A four-part
        // version is the ordinary case here — plenty of managers ship them.
        (
            "1.2.3.4",
            "1.2.3.4",
            true,
            "a four-part version matches itself",
        ),
        (
            "1.2.3.4",
            "1.2.3.5",
            false,
            "a four-part version that differs",
        ),
    ] {
        assert_eq!(
            resolver.satisfies_constraint(version, constraint),
            want,
            "`{version}` against `{constraint}`: {why}"
        );
    }
}

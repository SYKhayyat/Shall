use super::*;

#[test]
fn an_unknown_backend_does_not_outrank_your_distro() {
    // The onboarder can add backends at runtime, and nothing says a custom one is safe
    // to prefer over the distro's own build.
    let out = starter_order(&["weird".into(), "apt".into()]);
    assert_eq!(out, ["apt", "weird"]);
}

use super::*;

/// And the half that keeps attribution honest: a manager talking about one package is not
/// talking about another. This is what stops a `sync` that failed on a pre-existing wedge
/// from withdrawing the good line the command just wrote.
#[test]
fn attribution_does_not_spread_to_a_line_the_manager_never_named() {
    let message = "`npm` failed (exit 1): 404 Not Found - GET \
                   https://registry.npmjs.org/shall-no-such-pkg-zzz-9";
    assert!(mentions_package(message, "shall-no-such-pkg-zzz-9"));
    assert!(!mentions_package(message, "cowsay"));
}

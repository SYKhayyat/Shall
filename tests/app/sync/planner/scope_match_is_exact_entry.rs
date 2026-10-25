use super::*;

#[test]
fn scope_match_is_exact_entry() {
    let scopes = |names: &[&str]| -> Vec<String> { names.iter().map(|s| s.to_string()).collect() };

    assert!(ChangePlanner::in_scope(
        &scopes(&["module:dev"]),
        "module:dev"
    ));

    // Never a substring: `--module dev` must not sweep up `dev-tools`, and a scoped
    // upgrade acting on a package nobody named is the shape of the bug this repo is
    // named for.
    assert!(!ChangePlanner::in_scope(
        &scopes(&["module:dev-tools"]),
        "module:dev"
    ));
    assert!(!ChangePlanner::in_scope(
        &scopes(&["module:dev"]),
        "module:dev-tools"
    ));

    // A package belongs to every scope that declared it: the module that holds it and
    // the profile that reaches it. These were one `;`-joined string until the spec's
    // options became the grammar's own type — and a module whose name contained a
    // semicolon was two scopes to whoever split it.
    let both = scopes(&["module:dev", "profile:Work"]);
    assert!(ChangePlanner::in_scope(&both, "profile:Work"));
    assert!(ChangePlanner::in_scope(&both, "module:dev"));
    assert!(!ChangePlanner::in_scope(&both, "profile:Home"));

    // The name a delimiter could not carry. It is one scope, and it matches itself and
    // nothing else.
    let awkward = scopes(&["module:a;b"]);
    assert!(ChangePlanner::in_scope(&awkward, "module:a;b"));
    assert!(!ChangePlanner::in_scope(&awkward, "module:a"));
    assert!(!ChangePlanner::in_scope(&awkward, "b"));
}

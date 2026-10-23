use super::*;

#[test]
fn a_verb_never_shadows_a_builtin() {
    let v = verbs(&[("sync", &["upgrade"])]);
    // `sync` is a real command, so the verb is invisible and normal parsing proceeds.
    assert!(plan_user_verb(&argv(&["shall", "sync"]), &v, &builtins()).is_none());
}

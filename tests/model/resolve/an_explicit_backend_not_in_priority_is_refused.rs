use super::*;

#[test]
fn an_explicit_backend_not_in_priority_is_refused() {
    // The refusal has to name `priority`: a backend Shall simply cannot see and one
    // the host deliberately does not list produce the same silence otherwise.
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "snap:foo\n")],
    );
    let err = resolve(&f).unwrap_err();
    assert!(err.what.contains("isn't in your priority list"), "{}", err);
}

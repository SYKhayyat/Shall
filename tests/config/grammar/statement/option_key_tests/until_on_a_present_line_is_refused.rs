use super::*;

#[test]
fn until_on_a_present_line_is_refused() {
    // II.2: `@until` is for `absent:` only (absent now, present after). On a present line
    // it means "install this later", which nothing can act on. It used to parse clean.
    let err = parse_line("apt:steam@until=2026-07-20T00:00").unwrap_err();
    assert!(err.what.contains("only for `absent:`"), "{}", err);
    assert!(
        err.hint.unwrap().contains("@expires"),
        "must point at the present-line form"
    );
}

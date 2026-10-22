use super::*;

#[test]
fn a_repeated_single_value_option_is_refused_rather_than_resolved_by_order() {
    // `one()` returns `first()`, so a second value used to lose in silence. The hand-edited
    // hash bump is the shape that matters: `@sha256=<old>,sha256=<new>` verified against the
    // stale hash. Every single-valued key refuses a repeat at the same gate now.
    let err = p("appimage:https://example.com/t.AppImage@sha256=aaa,sha256=bbb").unwrap_err();
    assert!(err.to_string().contains("takes one value"), "{err}");
    let err = p("github:sharkdp/fd@bin=a,bin=b").unwrap_err();
    assert!(err.to_string().contains("takes one value"), "{err}");
    // One is still one.
    assert!(p("appimage:https://example.com/t.AppImage@sha256=aaa").is_ok());
    assert!(p("github:sharkdp/fd@bin=fd,formats=deb").is_ok());
}

use super::*;

/// `@template=` is a flag with a value: anything but `true` or `false` would fall
/// through to the plain-symlink mode carrying a word nobody acts on.
#[test]
fn a_link_template_that_is_neither_on_nor_off_is_refused() {
    for bad in ["maybe", "yes", "1", ""] {
        let line = format!("link:/a/b@target=/c,template={}", bad);
        assert!(p(&line).is_err(), "{} was accepted", line);
    }
    assert!(p("link:/a/b@target=/c,template=false").is_ok());
    assert!(p("link:/a/b@target=/c,template").is_ok());
}

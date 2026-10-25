use super::*;

/// AU10: `@version=` parsed, produced `"version": ""`, and exited 0 — while `npm:` with an
/// empty name was refused. Both spellings of the mistake, and the flag form that still has
/// to work, because a refusal that also catches `@hold` would have broken every pin.
#[test]
fn an_option_with_nothing_after_the_equals_is_refused() {
    for line in ["version=", "version=  ", "hold=", "arch=x86,extra="] {
        let Err(err) = parse_short(&o(), line) else {
            panic!("`@{}` was accepted", line);
        };
        assert!(
            err.what.contains("nothing after the `=`"),
            "`{}` was refused for the wrong reason: {}",
            line,
            err.what
        );
    }

    let err = parse_block_line(&o(), "after_install =").unwrap_err();
    assert!(err.what.contains("nothing after the `=`"), "{:?}", err);

    // Controls: the flag form and a real value are untouched.
    assert_eq!(parse_short(&o(), "hold").unwrap().one("hold"), Some("true"));
    assert_eq!(
        parse_short(&o(), "version=1.6").unwrap().one("version"),
        Some("1.6")
    );
}

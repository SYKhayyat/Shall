use super::*;

#[test]
fn the_rhai_marker_is_not_handed_to_the_engine_but_the_shebang_is() {
    // `#rhai` is a Shall marker; `#!` is the script's own first instruction. Confusing the
    // two either way breaks a dialect: a kept `#rhai` is a syntax error on line 1, and a
    // stripped `#!` leaves nothing to name the interpreter the author chose.
    let (dialect, body) = Dialect::of("#rhai\nlet x = 1;\n");
    assert!(matches!(dialect, Dialect::Rhai));
    assert!(!body.contains("#rhai"), "the marker survived: {:?}", body);

    let (dialect, body) = Dialect::of("#!/bin/sh\necho hi\n");
    assert!(matches!(dialect, Dialect::Process));
    assert!(body.starts_with("#!/bin/sh"), "the shebang was stripped");

    let (dialect, body) = Dialect::of("print('hi')");
    assert!(matches!(dialect, Dialect::Lua));
    assert_eq!(body, "print('hi')");
}

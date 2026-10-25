use super::*;

#[test]
fn an_option_the_backend_cannot_read_is_an_error_here_too() {
    // VIII.4: silently ignoring an option is how a config grows lines that do nothing.
    let err = parse("apt {\n  formats = deb\n}\n").unwrap_err();
    assert!(err.to_string().contains("apt"), "{}", err);

    let err = parse("github {\n  channel = stable\n}\n").unwrap_err();
    assert!(err.to_string().contains("channel"), "{}", err);
}

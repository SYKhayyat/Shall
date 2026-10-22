use super::*;

#[test]
fn a_regex_must_say_which_backend() {
    let err = p("re:^fonts-").unwrap_err();
    assert!(
        err.what.contains("does not say which backend"),
        "{}",
        err.what
    );
}

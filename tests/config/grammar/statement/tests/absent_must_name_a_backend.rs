use super::*;

#[test]
fn absent_must_name_a_backend() {
    // `absent:` reaches outside what Shall manages, so it cannot be left to `priority`.
    let err = p("absent:libreoffice").unwrap_err();
    assert!(err.what.contains("does not name a backend"), "{}", err.what);
}

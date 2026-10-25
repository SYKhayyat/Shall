use super::*;

#[test]
fn a_quoted_reading_equals_the_bare_value() {
    // `gsettings get` prints `'prefer-dark'`; the user wrote `prefer-dark`. Same value.
    assert!(already_set("'prefer-dark'", "prefer-dark"));
    assert!(already_set("prefer-dark\n", "prefer-dark"));
    assert!(already_set("true", "true"));
    // A registry read prints `"Dark"`; the same rule covers it.
    assert!(already_set("\"Dark\"", "Dark"));
}

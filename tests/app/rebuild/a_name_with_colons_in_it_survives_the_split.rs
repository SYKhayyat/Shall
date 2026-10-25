use super::*;

/// `split(':').next_back()` degraded this to `//host/x.tar.gz` and matched nothing —
/// and never checked the prefix named a backend at all.
#[test]
fn a_name_with_colons_in_it_survives_the_split() {
    let t = &targets(&["web:https://host/x.tar.gz"])[0];
    assert_eq!(t.backend.as_deref(), Some("web"));
    assert_eq!(t.name, "https://host/x.tar.gz");
}

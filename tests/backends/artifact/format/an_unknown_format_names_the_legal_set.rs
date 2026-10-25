use super::*;

#[test]
fn an_unknown_format_names_the_legal_set() {
    let err = Format::parse("snap").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("snap"));
    for f in Format::ALL {
        assert!(msg.contains(f.as_str()), "vocabulary missing {}", f);
    }
}

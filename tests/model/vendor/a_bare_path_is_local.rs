use super::*;

#[test]
fn a_bare_path_is_local() {
    assert!(matches!(
        Source::classify("./shared/modules"),
        Some(Source::Local(_))
    ));
    assert!(matches!(
        Source::classify("/home/a/repo"),
        Some(Source::Local(_))
    ));
    assert_eq!(Source::classify("   "), None);
}

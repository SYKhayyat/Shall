use super::*;

#[test]
fn with_nothing_set_the_source_is_the_default() {
    let resolved = resolve_root(None, &Settings::default()).unwrap();
    assert_eq!(resolved.source, RootSource::Default);
}

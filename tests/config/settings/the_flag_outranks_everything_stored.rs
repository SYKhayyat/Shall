use super::*;

#[test]
fn the_flag_outranks_everything_stored() {
    let settings = Settings {
        config_root: Some(at(&absolute("stored"))),
    };
    let flagged = at(&absolute("flagged"));
    let resolved = resolve_root(Some(&flagged), &settings).unwrap();
    assert_eq!(resolved.path, flagged);
    assert_eq!(resolved.source, RootSource::Flag);
}

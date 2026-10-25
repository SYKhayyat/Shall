use super::*;

/// A missing file is the ordinary case, not a refusal: nothing is approved and nothing
/// needs to be.
#[test]
fn no_definition_file_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, exec) = mock_exec();
    let mut reg = BackendRegistry::new();
    assert_eq!(
        load_custom_backends_from(
            &mut reg,
            &exec,
            &tmp.path().join("backends.toml"),
            &tmp.path().join("locks"),
        ),
        0
    );
}

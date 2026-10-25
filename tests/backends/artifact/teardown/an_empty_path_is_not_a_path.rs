use super::*;

/// The empty string is how all three records spell *no such path*, and deleting it would
/// be a delete of the current directory.
#[test]
fn an_empty_path_is_not_a_path() {
    let d = Deployed::default()
        .path("")
        .path("/tmp/real")
        .maybe_path(None);
    assert_eq!(d.paths, vec!["/tmp/real".to_string()]);
}

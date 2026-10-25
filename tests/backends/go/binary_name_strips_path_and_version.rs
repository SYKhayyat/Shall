use super::*;

#[test]
fn binary_name_strips_path_and_version() {
    let expected = if cfg!(windows) { "fzf.exe" } else { "fzf" };
    assert_eq!(
        GoBackendCore::binary_name("github.com/junegunn/fzf@latest"),
        expected
    );
    assert_eq!(GoBackendCore::binary_name("fzf"), expected);
}

use super::*;

#[test]
fn the_module_path_goes_behind_the_terminator() {
    assert_eq!(
        install_argv("github.com/junegunn/fzf@latest"),
        ["install", "--", "github.com/junegunn/fzf@latest"]
    );
}

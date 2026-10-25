use super::*;

#[test]
fn github_with_explicit_path_and_ref() {
    let (url, name) =
        resolve_module_source("github:acme/dotfiles@v2/modules/editors.module.txt").unwrap();
    assert_eq!(
        url,
        "https://raw.githubusercontent.com/acme/dotfiles/v2/modules/editors.module.txt"
    );
    assert_eq!(name, "editors");
}

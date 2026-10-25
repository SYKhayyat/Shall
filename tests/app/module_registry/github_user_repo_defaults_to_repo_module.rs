use super::*;

#[test]
fn github_user_repo_defaults_to_repo_module() {
    let (url, name) = resolve_module_source("github:acme/rust-dev").unwrap();
    assert_eq!(
        url,
        "https://raw.githubusercontent.com/acme/rust-dev/HEAD/rust-dev.module.txt"
    );
    assert_eq!(name, "rust-dev");
}

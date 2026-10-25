use super::*;

#[test]
fn a_bare_tilde_is_the_home_directory_and_is_inside_it() {
    let home = dirs::home_dir().unwrap();
    assert_eq!(resolve_target("~").unwrap(), home);
    assert_eq!(
        resolve_target("~/.gitconfig").unwrap(),
        home.join(".gitconfig")
    );
    assert!(!is_outside_home(&resolve_target("~/.config/nvim").unwrap()));
}

use super::*;

#[test]
fn guix_search_pairs_name_and_version() {
    let out =
        "name: hello\nversion: 2.12\nsynopsis: Hello, GNU world\n\nname: emacs\nversion: 29.1\n";
    let pkgs = guix_search(out, "guix");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "hello");
    assert_eq!(pkgs[0].version.as_deref(), Some("2.12"));
    assert_eq!(pkgs[1].name, "emacs");
    assert_eq!(pkgs[1].version.as_deref(), Some("29.1"));
}

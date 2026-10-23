use super::*;

#[test]
fn guix_list_via_ws_name_version() {
    // `guix package -I` is tab-separated name<TAB>version<TAB>outputs<TAB>path.
    let out = "hello\t2.12\tout\t/gnu/store/xxx\nemacs\t29.1\tout\t/gnu/store/yyy\n";
    let pkgs = ws_name_version(out, "guix").expect("this fixture parses");
    assert_eq!(pkgs[0].name, "hello");
    assert_eq!(pkgs[0].version.as_deref(), Some("2.12"));
    assert_eq!(pkgs[1].name, "emacs");
}

use super::*;

#[test]
fn bare_names_strips_the_architecture_qualifier() {
    // showmanual prints `libc6:i386` on a multi-arch host while dpkg-query prints the
    // bare `libc6`. Keeping the suffix records a package nothing can ever match.
    let pkgs = parse_bare_names("libc6:i386\n", "apt").expect("this fixture parses");
    assert_eq!(pkgs[0].name, "libc6");
}

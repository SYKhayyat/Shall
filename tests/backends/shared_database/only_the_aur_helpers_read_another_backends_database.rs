use super::*;

/// Both directions, because the table is only useful if the default is "its own".
#[test]
fn only_the_aur_helpers_read_another_backends_database() {
    assert_eq!(package_database("yay"), "pacman");
    assert_eq!(package_database("paru"), "pacman");
    for b in [
        "pacman", "apt", "dnf", "apk", "zypper", "xbps", "npm", "pnpm", "pip", "pipx", "cargo",
        "brew", "winget",
    ] {
        assert!(
            owns_its_database(b),
            "{} was folded into another backend",
            b
        );
        assert_eq!(package_database(b), b);
    }
}

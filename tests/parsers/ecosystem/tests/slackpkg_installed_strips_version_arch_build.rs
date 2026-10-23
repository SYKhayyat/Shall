use super::*;

#[test]
fn slackpkg_installed_strips_version_arch_build() {
    let out = "bash-5.1.016-x86_64-4\naaa_base-15.0-x86_64-3\nvim-9.0.2000-x86_64-1\n";
    let pkgs = slackpkg_installed(out, "slackpkg").expect("this fixture parses");
    assert_eq!(pkgs[0].name, "bash");
    assert_eq!(pkgs[1].name, "aaa_base");
    assert_eq!(pkgs[2].name, "vim");
}

use super::*;

#[test]
fn flatpak_refs_come_after_the_terminator() {
    let argv = install_argv(
        &["--system"],
        &[spec_with("org.gimp.GIMP", &[]), spec_with("--user", &[])],
    );
    assert_eq!(
        argv,
        [
            "install",
            "--system",
            "-y",
            "--noninteractive",
            "--or-update",
            "--",
            "org.gimp.GIMP",
            "--user"
        ]
    );
}

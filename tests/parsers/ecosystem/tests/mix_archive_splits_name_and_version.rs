use super::*;

#[test]
fn mix_archive_splits_name_and_version() {
    let out = "* hex-2.0.6\n* phx_new-1.7.0\n";
    let pkgs = mix_archive(out, "mix").expect("this fixture parses");
    assert_eq!(pkgs[0].name, "hex");
    assert_eq!(pkgs[0].version.as_deref(), Some("2.0.6"));
    assert_eq!(pkgs[1].name, "phx_new");
    assert_eq!(pkgs[1].version.as_deref(), Some("1.7.0"));
}

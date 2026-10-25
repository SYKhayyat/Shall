use super::*;

#[test]
fn everything_installed_is_nothing_to_report() {
    let out = "\
nvidia/550.78, 6.8.0-35-generic, x86_64: installed
v4l2loopback/0.12.7, 6.8.0-35-generic, x86_64: installed
";
    assert!(not_installed(out).is_empty());
}

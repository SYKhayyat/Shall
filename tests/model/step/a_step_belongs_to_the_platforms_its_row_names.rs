use super::*;

/// The OS filter decides what a machine can name, and it is asserted from both sides so the
/// Windows arm is exercised on Linux and back.
#[test]
fn a_step_belongs_to_the_platforms_its_row_names() {
    let rows = rows();
    let fwupd = rows
        .iter()
        .find(|s| s.name == "fwupd")
        .expect("fwupd is shipped");
    assert!(fwupd.applies_to("linux"));
    assert!(!fwupd.applies_to("windows"));
    assert!(!fwupd.applies_to("macos"));

    let rustup = rows
        .iter()
        .find(|s| s.name == "rustup")
        .expect("rustup is shipped");
    for os in ["linux", "windows", "macos"] {
        assert!(rustup.applies_to(os), "rustup should exist on {os}");
    }
}

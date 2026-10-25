use super::*;

#[test]
fn a_backend_that_installs_one_artifact_gets_no_formats_line() {
    assert_eq!(format_choice("apt", &opts(&[])), None);
    assert_eq!(format_choice("cargo", &opts(&[("formats", "deb")])), None);
}

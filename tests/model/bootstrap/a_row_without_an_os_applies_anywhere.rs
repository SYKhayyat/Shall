use super::*;

#[test]
fn a_row_without_an_os_applies_anywhere() {
    assert!(row("brew", None).applies_to("linux"));
    assert!(row("brew", None).applies_to("windows"));
}

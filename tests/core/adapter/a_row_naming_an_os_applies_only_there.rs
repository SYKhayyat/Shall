use super::*;

/// The question every table asked separately, asked once — and asked for a platform this
/// test is not running on, which is what taking the OS as a parameter buys.
#[test]
fn a_row_naming_an_os_applies_only_there() {
    assert!(row("ufw", None).applies_to("linux"));
    assert!(row("ufw", None).applies_to("windows"));
    assert!(row("ufw", Some("linux")).applies_to("linux"));
    assert!(!row("ufw", Some("linux")).applies_to("windows"));
    // Case-insensitive: the field is user-written.
    assert!(row("ufw", Some("Linux")).applies_to("linux"));
}

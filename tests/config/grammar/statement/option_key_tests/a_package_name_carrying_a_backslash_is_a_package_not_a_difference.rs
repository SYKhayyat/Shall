use super::*;

#[test]
fn a_package_name_carrying_a_backslash_is_a_package_not_a_difference() {
    // `winget list` reports 185 such names out of 278 on a stock Windows box. The typed
    // prefixes were shielded from `looks_like_expression` and the package line — the most
    // common line in the language — was not.
    for line in [
        r"winget:ARP\Machine\X64\Firefox",
        r"winget:MSIX\Microsoft.AV1VideoExtension_2.0.24.0_x64__8wekyb3d8bbwe",
    ] {
        assert!(
            matches!(parse_line(line).unwrap(), Statement::Package(_)),
            "{line} must be a package line"
        );
    }
}

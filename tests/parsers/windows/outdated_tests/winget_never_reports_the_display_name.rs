use super::*;

/// The display Name is not an identifier `winget install` accepts, and a declaration is
/// written with the Id. Reporting `7-Zip 25.01 (x64)` would name a package nothing has.
#[test]
fn winget_never_reports_the_display_name() {
    assert!(!parse_winget_outdated(WINGET)
        .iter()
        .any(|p| p.name.contains(' ')));
}

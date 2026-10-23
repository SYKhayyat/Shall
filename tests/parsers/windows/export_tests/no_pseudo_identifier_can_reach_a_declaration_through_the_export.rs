use super::*;

/// The whole point of the change: these are what `winget list` reports and `winget install`
/// refuses, and the export is where they are absent. If a pseudo-id ever appears in an
/// export, adoption would plant a declaration that can never converge (Q36).
#[test]
fn no_pseudo_identifier_can_reach_a_declaration_through_the_export() {
    let pkgs = parse_winget_export(EXPORT).expect("this fixture parses");
    assert!(
        !pkgs
            .iter()
            .any(|p| p.name.starts_with("ARP\\") || p.name.starts_with("MSIX\\")),
        "an ARP/MSIX identifier reached the adopted set"
    );
}

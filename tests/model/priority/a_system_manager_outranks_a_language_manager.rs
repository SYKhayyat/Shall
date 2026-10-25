use super::*;

#[test]
fn a_system_manager_outranks_a_language_manager() {
    // V.14: if both apt and cargo have ripgrep, apt wins — your distro maintains that
    // build and updates it with everything else.
    let out = starter_order(&["cargo".into(), "npm".into(), "apt".into()]);
    assert_eq!(out, ["apt", "cargo", "npm"]);
}

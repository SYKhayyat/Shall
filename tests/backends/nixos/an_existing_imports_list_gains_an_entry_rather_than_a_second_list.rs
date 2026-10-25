use super::*;

/// The common real shape: a machine that already imports its hardware configuration. The new
/// path joins that list rather than opening a second `imports`, which nix rejects as a
/// duplicate attribute.
#[test]
fn an_existing_imports_list_gains_an_entry_rather_than_a_second_list() {
    let with_hw = "{ config, pkgs, ... }:
{
  imports = [ ./hardware-configuration.nix ];
  system.stateVersion = \"26.05\";
}
";
    let out = with_import(with_hw, GENERATED).expect("editable");
    assert_eq!(
        out.matches("imports").count(),
        1,
        "a second imports list:
{out}"
    );
    assert!(out.contains("hardware-configuration.nix"), "{out}");
    assert!(out.contains(GENERATED), "{out}");
}

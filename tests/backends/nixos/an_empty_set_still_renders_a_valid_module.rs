use super::*;

/// An empty model is still a valid module, not an empty file. A truncated import would fail
/// the user's whole rebuild, and "you removed your last package" must not do that.
#[test]
fn an_empty_set_still_renders_a_valid_module() {
    let out = render(&Module::default()).expect("renders");
    assert!(
        out.contains("environment.systemPackages = with pkgs; [\n  ];"),
        "{out}"
    );
    assert!(out.contains("{ config, lib, pkgs, ... }:"), "{out}");
    // Nothing declared is nothing written: an absent `networking.firewall.enable` leaves
    // NixOS's own default alone, and writing `false` would silently open the machine.
    assert!(!out.contains("networking.firewall"), "{out}");
}

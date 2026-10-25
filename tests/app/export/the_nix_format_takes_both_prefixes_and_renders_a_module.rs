use super::*;

/// `J5`: the nix format takes BOTH prefixes, and renders through the backend rather than
/// through a second copy of the format.
#[test]
fn the_nix_format_takes_both_prefixes_and_renders_a_module() {
    assert_eq!(Format::parse("nix"), Some(Format::Nix));
    assert_eq!(Format::parse("nixos"), Some(Format::Nix));
    assert_eq!(Format::Nix.filename(), "shall-packages.nix");

    let out = Format::Nix
        .render(&[
            p("nix", "ripgrep", Some("14.1.0")),
            p("nixos", "jq", None),
            p("apt", "curl", None),
        ])
        .expect("renders")
        .expect("two nix packages apply");

    // A NixOS module, not a list of names.
    assert!(
        out.contains("environment.systemPackages = with pkgs; ["),
        "{out}"
    );
    assert!(out.contains("ripgrep"), "{out}");
    assert!(out.contains("jq"), "{out}");
    // The apt package is somebody else's format.
    assert!(!out.contains("curl"), "{out}");
    // **No version.** A NixOS module pins by pinning nixpkgs, not per package — the same
    // reason `nixos:` refuses an `@version=`. A rendered `14.1.0` would not evaluate.
    assert!(!out.contains("14.1.0"), "{out}");
}

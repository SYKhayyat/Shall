use super::*;

/// **A name this cannot prove safe is refused, not interpolated.** The generated file is
/// evaluated as root by `nixos-rebuild`; a package or service name carrying Nix syntax
/// would rewrite what the rebuild evaluates. Names are only ever identifiers here —
/// `render` refuses everything else by name rather than escaping and hoping.
#[test]
fn a_name_carrying_nix_syntax_is_refused_not_interpolated() {
    let hostile_packages = packages(&["hello); boot.kernelParams = [ \"x\" ]; #"]);
    assert!(
        render(&hostile_packages).is_err(),
        "a package name with \
         statement syntax must not reach the generated file"
    );

    let mut hostile_services = Module::default();
    hostile_services.services.insert(
        "nginx.enable = true; security.sudo.wheelNeedsPassword = false; #".to_string(),
        true,
    );
    assert!(render(&hostile_services).is_err());

    // The controls: every legitimate spelling still renders. Dotted attrpaths are how
    // real package sets are named (`nodePackages.typescript`), hyphens are in almost
    // every package name, and services can carry sub-options.
    let legit = packages(&["ripgrep", "nodePackages.typescript", "python311"]);
    assert!(render(&legit).is_ok());
    let mut dotted_service = Module::default();
    dotted_service
        .services
        .insert("nginx.virtualHosts.default".to_string(), true);
    assert!(render(&dotted_service).is_ok());
}

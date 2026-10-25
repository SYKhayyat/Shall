use super::*;

/// A machine with no nix packages writes no nix file, rather than an empty module that
/// would import cleanly and silently declare nothing.
#[test]
fn no_nix_packages_means_no_nix_file() {
    assert!(matches!(
        Format::Nix.render(&[p("apt", "curl", None)]),
        Ok(None)
    ));
}

use super::*;

#[test]
fn purl_and_ecosystem_maps() {
    assert_eq!(purl_type("cargo"), Some("cargo"));
    assert_eq!(purl_type("pnpm"), Some("npm"));
    assert_eq!(purl_type("pipx"), Some("pypi"));
    assert_eq!(purl_type("winget"), None);
    assert_eq!(osv_ecosystem("pip"), Some("PyPI"));
    assert_eq!(osv_ecosystem("cargo"), Some("crates.io"));
    assert_eq!(osv_ecosystem("snap"), None);
    assert_eq!(
        purl("cargo", "ripgrep", Some("13.0.0")).as_deref(),
        Some("pkg:cargo/ripgrep@13.0.0")
    );
}

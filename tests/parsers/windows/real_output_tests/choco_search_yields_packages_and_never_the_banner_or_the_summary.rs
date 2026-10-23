use super::*;

/// choco's own banner and its own summary line were becoming packages: a package named
/// `Chocolatey` at version `v2.7.3`, and one named `5` at version `packages`. Neither is a
/// package and both were offered to a user choosing what to install.
#[test]
fn choco_search_yields_packages_and_never_the_banner_or_the_summary() {
    for (case, out) in [("human", CHOCO_HUMAN), ("machine", CHOCO_MACHINE)] {
        let names: Vec<String> = parse_choco_search(out)
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert!(
            names.iter().any(|n| n == "ripgrep"),
            "{case}: lost the real package: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.eq_ignore_ascii_case("chocolatey")),
            "{case}: the version banner became a package: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "5"),
            "{case}: the `N packages found.` summary became a package: {names:?}"
        );
    }
}

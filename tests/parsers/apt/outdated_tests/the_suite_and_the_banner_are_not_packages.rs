use super::*;

/// The suite list is not part of the name, and `Listing...` is not a package.
#[test]
fn the_suite_and_the_banner_are_not_packages() {
    let p = parse_apt_outdated(APT);
    assert!(!p.iter().any(|x| x.name.contains('/')), "{:?}", p);
    assert!(!p.iter().any(|x| x.name.starts_with("Listing")), "{:?}", p);
}

use super::*;

/// **Synthetic, and labelled as such — this is the test the rule 250 lines below condemns.**
///
/// Its input was typed by hand and labelled `"spack"`, and `names_only` is the *installed*
/// lister for `opam` and `emerge`. The rule at the bottom of this file was written about
/// exactly this shape: *"a parser is tested against output captured from the tool it parses,
/// and from no other tool. `names_only` serves five managers and its only test used a spack
/// fixture — it passed, and said nothing whatever about pixi, which is exactly where it was
/// wrong."*
///
/// It is kept because what it asserts is real and cheap — a header row and a dashed rule are
/// not packages, which is a property of the *function* rather than of any manager. It is
/// renamed so that nobody reads it as coverage of a manager. The coverage lives in
/// `tests/parsers/installed_listing_fixture_tests.rs`, against output captured from the tools
/// themselves.
#[test]
fn names_only_drops_furniture_synthetic_not_a_managers_output() {
    let out = "Package\n----------\nripgrep\nfd\n\n";
    let pkgs = names_only(out, "spack").expect("this fixture parses");
    assert_eq!(
        pkgs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["ripgrep", "fd"]
    );
}

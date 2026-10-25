use super::*;

#[test]
fn cabal_names_an_absent_package_in_both_versions_phrasings() {
    let p = cabal();
    for out in [
        "cabal: Unknown package \"shall-no-such-pkg-zzz\".",
        "Error: [Cabal-7144]
Unknown package \"shall-no-such-pkg-zzz\". ",
    ] {
        assert!(
            p.names_an_absent_package(&ExitPolicy::haystack(out.as_bytes(), b"")),
            "cabal did not recognise its own words for a missing name: {out}"
        );
    }
}

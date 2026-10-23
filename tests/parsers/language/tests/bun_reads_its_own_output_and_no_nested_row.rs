use super::*;

/// The tool's own output, captured from `bun pm ls -g` and `bun pm ls -g --all` on the
/// machine this was written on. The second is the one with teeth: bun nests four levels
/// deep, and a parser that trims the tree glyphs off before reading reports every
/// dependency as a package someone installed.
#[test]
fn bun_reads_its_own_output_and_no_nested_row() {
    const FLAT: &str = include_str!("../../../../tests/fixtures/bun/ls-global.txt");
    const NESTED: &str = include_str!("../../../../tests/fixtures/bun/ls-global-all.txt");

    let flat = parse_bun_list(FLAT);
    assert_eq!(
        flat.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["cowsay"],
        "the header path line is not a package"
    );

    let nested = parse_bun_list(NESTED);
    let found = |n: &str, v: &str| {
        nested
            .iter()
            .any(|p| p.name == n && p.version.as_deref() == Some(v))
    };
    // `is-fullwidth-code-point` appears twice in that fixture: 2.0.0 hoisted to the top
    // level, 3.0.0 only as a child of `string-width`. Exactly one of them is installed.
    assert!(found("is-fullwidth-code-point", "2.0.0"), "{nested:?}");
    assert!(
        !found("is-fullwidth-code-point", "3.0.0"),
        "a row nested under `string-width@4.2.3` was reported as a global install"
    );
    assert!(!found("ansi-regex", "5.0.1"), "nested under `strip-ansi`");
}

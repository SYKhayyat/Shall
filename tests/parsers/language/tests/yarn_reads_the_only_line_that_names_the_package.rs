use super::*;

/// Verbatim from `yarn global list` on a host with `catj` installed, node's deprecation
/// warning and all — the output a parser has to survive, not the one it would like.
///
/// Before this, `parse_yarn_list` dropped every line containing `info` and looked for an
/// ASCII tree. yarn prints no tree, and the `info` line is the only one carrying the
/// package. So `shall list -b yarn` was empty on a machine with yarn packages installed,
/// and `remove` — which is gated on `info`, reading the same listing — could not remove
/// them either.
#[test]
fn yarn_reads_the_only_line_that_names_the_package() {
    let real = "yarn global v1.22.22
                (node:12864) [DEP0169] DeprecationWarning: `url.parse()` behavior is not standardized
                info \"catj@1.0.4\" has binaries:
   - catj
Done in 0.07s.
";
    let r = parse_yarn_list(real);
    assert_eq!(r.len(), 1, "{:?}", r);
    assert_eq!(r[0].name, "catj");
    assert_eq!(r[0].version.as_deref(), Some("1.0.4"));

    // A package with no binaries is still a package, and a scoped name keeps its own `@`.
    let more = "info \"@babel/cli@7.24.1\" has no binaries
info \"left-pad@1.3.0\" has no binaries
";
    let r = parse_yarn_list(more);
    assert_eq!(
        r.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["@babel/cli", "left-pad"]
    );
    assert_eq!(r[0].version.as_deref(), Some("7.24.1"));

    // yarn's other chatter carries no quoted spec and must not become a package. An
    // invented name is worse than a missing one: it reads as drift and schedules a removal.
    let chatter = "yarn global v1.22.22
info Visit https://yarnpkg.com/en/docs/cli/global for documentation
Done in 0.05s.
";
    assert!(parse_yarn_list(chatter).is_empty());
}

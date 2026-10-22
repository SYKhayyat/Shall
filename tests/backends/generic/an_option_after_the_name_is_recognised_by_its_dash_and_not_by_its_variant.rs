use super::*;

/// Whether the `--` terminator survives is read off the tokens, never off a label.
///
/// The bug this replaces: `Flag`, `TrailingPositional` and `RequiredFlag` built identical
/// argv and only their names decided the terminator, so `luarocks` and `mix` — both
/// carrying a bare positional version — lost it on pinned installs while keeping it on
/// unpinned ones. Same command, same tool, protection that came and went with whether
/// someone had written a version on the line.
#[test]
fn an_option_after_the_name_is_recognised_by_its_dash_and_not_by_its_variant() {
    // Options: the terminator cannot precede these.
    assert!(VersionPin::after(vec!["-v".into(), "{version}".into()]).emits_trailing_option());
    assert!(
        VersionPin::after(vec!["--version".into(), "{version}".into()]).emits_trailing_option()
    );
    // Operands: it can, and does. luarocks, mix and pub all measured in the `tools` image
    // on 2026-08-04 — `luarocks install -- <rock> <version>` and
    // `dart pub global activate -- <pkg> <version>` produce output identical to the same
    // command without the terminator, and `luarocks install --` answers
    // `Error: missing argument 'rock'` with usage `<rock> [<version>]`.
    assert!(!VersionPin::after(vec!["{version}".into()]).emits_trailing_option());
    assert!(
        !VersionPin::after_required(vec!["{version}".into()], "latest").emits_trailing_option()
    );
    // A required version does not change the answer — only the token does.
    assert!(
        VersionPin::after_required(vec!["-v".into(), "{version}".into()], "latest")
            .emits_trailing_option()
    );
    // Neither of the other placements puts anything after the name.
    assert!(!VersionPin::Inline("{name}@{version}".into()).emits_trailing_option());
    assert!(
        !VersionPin::Before(vec!["--version".into(), "{version}".into()]).emits_trailing_option()
    );
}

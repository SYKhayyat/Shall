use super::*;

/// **npm and pnpm were the siblings this module never covered**, and the omission cost a
/// backend. The bytes are npm's own, captured in the fedora integration image on
/// 2026-08-10 with nothing installed globally — exit 0, and no `dependencies` key at all:
///
/// ```text
/// $ npm list -g --depth=0 --json
/// {
///   "name": "lib"
/// }
/// ```
///
/// `parse_npm_style_json` read that as unrecognised *deliberately*, to catch npm renaming
/// the key. What it caught was the ordinary state of a fresh Node install: `check health`
/// said `[FAIL] npm — says it is ready but cannot list`, npm left the READY set on every
/// machine with no global packages, and the fedora leg lost a real lifecycle to it — which
/// is how it was found, eleven days later, the first time CI could run.
///
/// The rule this module is named for was already written down twice. npm is the third.
#[test]
fn an_empty_npm_global_is_empty_and_not_unreadable() {
    let out = "{
  \"name\": \"lib\"
}
";
    assert_eq!(
        parse_installed("npm", out).expect("an empty npm is empty, not unreadable"),
        Vec::new()
    );
    // pnpm wraps the same shape in an array, and an empty one says the same thing.
    assert_eq!(
        parse_installed("pnpm", "[]").expect("an empty pnpm is empty, not unreadable"),
        Vec::new()
    );
    // And the key present but empty, which always parsed and must keep doing so.
    assert_eq!(
        parse_installed("npm", r#"{"name":"lib","dependencies":{}}"#).expect("parses"),
        Vec::new()
    );
}

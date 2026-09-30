//! May the planner read a file to decide whether a declaration is satisfied?
//!
//! **No, and the reason is that `in_effect` already answers that question for every `extras`
//! kind.** `link:`, `dir:`, `service:` and `setting:` are not packages: there is no manager to
//! ask, so "is this satisfied" is a question about the *machine*, and `apply::extras::in_effect`
//! is the one function that answers it — by asking, in the same order and through the same
//! resolution the installer uses, so a hand-edited destination reads as drift rather than as
//! `unverifiable` (B0b).
//!
//! **`template_needs_update` was a second answer, in the planner, and it was wrong.** It compared
//! `checksum_pair(spec.name, @target)` — the **raw** template on disk against the destination the
//! installer had **rendered**. A template with a placeholder in it therefore never matched, so
//! every `link:@template=true` line would have reported perpetual drift and scheduled a change on
//! every sync. The read-back half had already been fixed for exactly this (`#69`: render, then
//! compare, pinned by `a_rendered_template_is_read_back_rendered_not_raw`); the planning half was
//! the sibling that survived.
//!
//! **It was also unreachable**, which is why the bug is Low and not worse: `link`'s register
//! installs no `Queryable` capability, so `spec_is_missing` answers `Missing` at the capability
//! check and the branch below it never runs. That is a reason to delete it, not a reason to keep
//! it: the branch dies the moment `link` becomes queryable, and it dies *reporting drift on every
//! line*. `PLAN.md` #91 named both options and said not both — the second implementation is gone
//! and what remains is the single authority.
//!
//! **This is a prohibition with a floor, because a prohibition that cannot fail is a gate that
//! reports green having examined nothing.** The predicate is driven against the real file (which
//! must be clean) *and* against the deleted code verbatim (which must trip it), so the check is
//! known to be capable of failing before it is believed.

use std::path::PathBuf;

/// A read of the filesystem used to decide whether a declaration is satisfied.
fn reads_a_file_to_decide(contents: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (n, line) in contents.lines().enumerate() {
        let line = line.trim();
        // Comments are not code, and a comment *about* the deleted function must not keep the
        // rule alive in the reader's eye.
        if line.starts_with("//") {
            continue;
        }
        for needle in [
            "checksum_pair",
            "fs::read(",
            "fs::read_to_string",
            "read_dir",
            "File::open",
        ] {
            if line.contains(needle) {
                found.push(format!("{}: {line}", n + 1));
            }
        }
    }
    found
}

fn planner() -> String {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app/sync/planner.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("reading {}: {e}", p.display()))
}

#[test]
fn the_planner_asks_the_registry_and_never_the_filesystem() {
    let found = reads_a_file_to_decide(&planner());
    assert!(
        found.is_empty(),
        "src/app/sync/planner.rs reads a file to decide something ({} line(s)).\n\n\
         `in_effect` is the single authority for whether a `link:`/`dir:`/`service:`/`setting:` \
         line is satisfied, and it renders a template before comparing it. A second answer in \
         the planner is either dead or wrong: `template_needs_update` compared the RAW template \
         against the RENDERED destination, so every templated link would have reported drift for \
         ever. If this is a genuinely new question, give it a name and a test rather than a \
         second read.\n\n{}",
        found.len(),
        found.join("\n")
    );
}

/// **The control, and it is the whole test.** The deleted function, verbatim: if this does not
/// trip the predicate above, the predicate examines nothing and the test above is decoration.
#[test]
fn the_predicate_still_catches_the_code_it_was_written_for() {
    let deleted = r#"
    async fn template_needs_update(&self, spec: &PackageSpec) -> bool {
        let target = match spec.options.one("target") {
            Some(s) => Path::new(s),
            None => return true,
        };
        let source = Path::new(&spec.name);
        if !tokio::fs::try_exists(target).await.unwrap_or(false) {
            return true;
        }
        let (s_hash, t_hash) = crate::core::security::checksum_pair(source, target).await;
        match (s_hash, t_hash) {
            (Ok(s), Ok(t)) => s != t,
            _ => true,
        }
    }
"#;
    let found = reads_a_file_to_decide(deleted);
    assert_eq!(
        found.len(),
        1,
        "the predicate must catch `checksum_pair` in the code this test removed, and it caught \
         {found:?}"
    );

    // And the other direction, because a predicate that trips on everything is as useless as one
    // that trips on nothing: a comment *about* the deleted function is not a read.
    let prose = "// `template_needs_update` compared checksum_pair of the raw source; see #91.\n";
    assert!(
        reads_a_file_to_decide(prose).is_empty(),
        "the predicate is reading comments, and a comment about the bug keeps the rule alive"
    );
}

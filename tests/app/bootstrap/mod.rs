//! Mirrors `src/app/bootstrap.rs` — the artifact `shall export` renders (#66, #67, II.94).
//!
//! What these tests pin is the two ways the artifact can quietly stop being true: escalation
//! leaking the preview's `-n` into a script somebody runs with no credentials primed, and a
//! hole rendered as an ordinary line instead of a confession.

use super::*;
use shall::app::bootstrap::{Format, Omission, Script};

#[test]
fn an_escalated_command_carries_plain_sudo() {
    // The product runs `-n` because it primes credentials once per run (S88); a bootstrap
    // script is run by somebody who has not primed anything, and the first escalated command
    // there should be allowed to ask.
    let mut s = Script::default();
    s.command(
        "apt-get",
        vec!["install".into(), "-y".into(), "--".into(), "pv".into()],
        true,
    );
    let out = s.render(Format::Bash, "");
    assert!(out.contains("sudo apt-get install -y -- pv"), "{out}");
    assert!(
        !out.contains("sudo -n"),
        "the preview's `-n` leaked into the artifact"
    );
}

#[test]
fn an_argument_with_a_space_is_quoted_not_split() {
    // A single-quote arg is the one whose escaping rule is written down: a literal quote
    // becomes `'\''`. A generous rule is what breaks a script on the machine whose path has
    // a space in it — the arg survives intact or the test fails.
    let mut s = Script::default();
    s.command(
        "ln",
        vec!["-sfn".into(), "/a b".into(), "'$HOME'/.c".into()],
        false,
    );
    let out = s.render(Format::Bash, "");
    let expected = concat!("ln -sfn '/a b' ''", "\\", "''$HOME'", "\\", "''/.c'");
    assert!(out.contains(expected), "got:\n{out}");
}

#[test]
fn an_omission_is_rendered_inside_the_artifact() {
    let mut s = Script::default();
    s.command(
        "cargo",
        vec!["install".into(), "--".into(), "rg".into()],
        false,
    );
    s.omitted.push(Omission {
        line: "modules/dev.txt:12 exec:./gen".into(),
        why: "the script it runs lives in the config repo, not in this artifact".into(),
    });
    let out = s.render(Format::Bash, "");
    assert!(
        out.contains("What this script does NOT do"),
        "a lossy artifact did not say so:\n{out}"
    );
    assert!(
        out.contains("#   modules/dev.txt:12 exec:./gen — the script it runs lives in the config repo, not in this artifact"),
        "the hole was not marked by name:\n{out}"
    );
}

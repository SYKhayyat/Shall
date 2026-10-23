use super::*;

/// **Both of these were found by `shall list` on a real Windows box, 2026-08-07** — the run
/// that `LX-1` made capable of complaining. Before it, each was a silent empty listing, which
/// the planner reads as *"this machine has none of these"* and answers by installing every
/// declared package and dropping every removal.
///
/// They are opposite mistakes with the same symptom, which is why both are pinned here.
#[test]
fn an_empty_pipx_is_empty_and_not_unreadable() {
    // `pipx list --json` on a machine with nothing: a sentence, then a JSON document whose
    // `venvs` object is empty. Four lines that are neither prose nor package rows, so the
    // line count said "unread" about an answer the reader understood perfectly.
    let out = "nothing has been installed with pipx\x20
{
\"pipx_spec_version\": \"0.1\",
\"venvs\": {}
}
";
    assert_eq!(
        parse_installed("pipx", out).expect("an empty pipx is empty, not unreadable"),
        Vec::new()
    );
}

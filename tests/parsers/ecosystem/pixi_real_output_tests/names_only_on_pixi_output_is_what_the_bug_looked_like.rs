use super::*;

/// The red this fix was watched against, kept as evidence rather than as a memory.
/// `names_only` is still the right parser for opam, spack and emerge; routing pixi to it
/// was the defect, and this says what that cost on real output.
#[test]
fn names_only_on_pixi_output_is_what_the_bug_looked_like() {
    let junk: Vec<String> = names_only(SEARCH, "pixi")
        .expect("this fixture parses")
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert!(
        junk.len() > 10,
        "expected the old parser to emit a pile of junk, got {junk:?}"
    );
    // What it really emitted, verbatim: field labels, the separator, the ellipsis, and
    // six bare version numbers out of the "Other Versions" table.
    //
    // `Dependencies:` was on this list and is not any more: `is_noise_line` now drops a
    // line ending in a colon, because a heading that introduces a list is the manager's
    // prose and not a candidate package. That narrows the junk without narrowing the
    // finding — the pile below is still a pile, and none of it is `ripgrep`.
    for j in ["SHA256", "Timestamp", "License", "-", "...", "15.1.0"] {
        assert!(
            junk.iter().any(|n| n == j),
            "expected junk `{j}` in {junk:?}"
        );
    }
    assert!(
        !junk.iter().any(|n| n == "ripgrep"),
        "the old parser never even produced the right answer: {junk:?}"
    );
}

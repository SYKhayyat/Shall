use super::*;

/// Four cases, enumerated, because the interesting one was reachable and unwritten: a tool
/// that does not document the flag got the same "Add `@unverified`" advice as one that does,
/// and a user who *had* written `@unverified` got no explanation at all (N-4).
#[test]
fn advice_never_names_a_flag_the_tool_would_drop() {
    // The line does not opt out, and the tool accepts the flag: name it.
    let accepted = verification_note("helm", "helm", "--verify=false", false, false, REFUSAL)
        .expect("a verification refusal earns advice");
    assert!(
        accepted.contains("Add `@unverified`"),
        "the one case where the flag helps must name it: {accepted}"
    );

    // The line does not opt out, and the tool would drop the flag: do not name it.
    let withheld = verification_note("helm", "helm", "--verify=false", false, true, REFUSAL)
        .expect("a verification refusal still earns an explanation");
    assert!(
        !withheld.contains("Add `@unverified`"),
        "advised a flag this tool does not document, which Shall would withhold: {withheld}"
    );
    assert!(
        withheld.contains("--verify=false") && withheld.contains("not in its help"),
        "the explanation must say which flag is missing and from where: {withheld}"
    );

    // The line already opts out and the flag went out: the manager's own words stand.
    assert_eq!(
        verification_note("helm", "helm", "--verify=false", true, false, REFUSAL),
        None,
        "nothing to add when the flag was sent and the manager refused anyway"
    );

    // The line already opts out and the flag was withheld: say so, and do not ask for it.
    let already = verification_note("helm", "helm", "--verify=false", true, true, REFUSAL)
        .expect("a withheld flag must be explained to someone who asked for it");
    assert!(
        already.contains("already says `@unverified`"),
        "a user who wrote the flag must be told it never went out: {already}"
    );
    assert!(
        !already.contains("Add `@unverified`"),
        "asked the user for what they already wrote: {already}"
    );
}

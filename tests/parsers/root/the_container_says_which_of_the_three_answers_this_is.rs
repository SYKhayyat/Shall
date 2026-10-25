use super::*;

/// The three answers a structured reader has to keep apart, asserted as three.
///
/// The arm this replaces lived inside `or_unrecognised` and returned `Ok(found)` â€” empty â€”
/// for anything whose output held a parseable JSON document, whether or not the reader had
/// got a single package out of it. That is `Some(n)` and `None` both collapsed into
/// `Some(0)`, which is the collapse the whole `LX-1` type exists to prevent.
#[test]
fn the_container_says_which_of_the_three_answers_this_is() {
    let pkg = || vec![Package::new("jq", "test")];

    // Read something: always fine, whatever the container said.
    assert!(or_unrecognised_json("test", pkg(), Some(9), "what", "out").is_ok());
    assert!(or_unrecognised_json("test", pkg(), None, "what", "out").is_ok());

    // An empty container is an empty machine, which is a true and common answer.
    assert_eq!(
        or_unrecognised_json("test", vec![], Some(0), "what", "out").expect("empty is empty"),
        vec![]
    );

    // Entries offered, none read: a schema change.
    let err = or_unrecognised_json("test", vec![], Some(4), "no `name` in any entry", "out")
        .expect_err("four unread entries is not an empty machine");
    assert_eq!(err.data_lines, 4, "the count is of entries, not of lines");
    assert!(err.sample.starts_with("no `name` in any entry"));

    // No container at all: also a schema change, and the one a length cannot express.
    let err = or_unrecognised_json("test", vec![], None, "no `data` key", "a\nb\n\nc\n")
        .expect_err("a missing container is not an empty machine");
    assert_eq!(
        err.data_lines, 3,
        "with no entry count to report, the refusal falls back to the size of the answer"
    );
}

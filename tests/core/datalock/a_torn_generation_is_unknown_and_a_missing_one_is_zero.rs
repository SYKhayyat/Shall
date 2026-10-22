use super::*;

/// The counter that lets a reader detect a writer without waiting for one. It moves when a
/// writer *finishes*, so an unchanged count with no holder means the reader is strictly
/// after that writer rather than inside it.
/// **A torn generation file is unknown, not zero.**
///
/// It was read with `.ok().unwrap_or(0)`, and `0` is *lower* than any real generation
/// rather than an error â€” so two observations straddling a crashed writer compared equal
/// and `spans_one_moment` said a multi-file read saw one moment when it had not. A file
/// that has never been written is still a real `0`: nothing has committed yet.
#[test]
fn a_torn_generation_is_unknown_and_a_missing_one_is_zero() {
    let dir = tmp("generation-torn");
    std::fs::create_dir_all(&dir).unwrap();
    assert_eq!(
        observe(&dir).count,
        Some(0),
        "a data directory nothing has written to is at generation zero"
    );

    std::fs::write(dir.join(GENERATION_FILE), "not a number").unwrap();
    let torn = observe(&dir);
    assert_eq!(torn.count, None);
    assert!(
        !torn.spans_one_moment(observe(&dir)),
        "two unreadable counters are two unknowns, not one moment seen twice"
    );
}

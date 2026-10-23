/// A read and a mutation are bounded differently, and the reason is in the numbers: the
/// mutation bound exists for `Checkpoint-Computer`, silent for its whole run, and a read
/// takes seconds. Collapsing them back to one number is the change this pins against —
/// it reads as a simplification and costs fifteen minutes per wedged listing.
#[test]
fn a_read_is_bounded_tighter_than_a_mutation() {
    let read = super::RawExecutor::reader().idle;
    let write = super::RawExecutor::mutator().idle;
    let (Some(read), Some(write)) = (read, write) else {
        panic!("both layers must carry a bound by default: {read:?} / {write:?}");
    };
    assert!(
        read < write,
        "a read waits as long as a mutation ({read:?} vs {write:?})"
    );
    assert_eq!(read.as_secs(), super::DEFAULT_QUERY_IDLE_TIMEOUT_SECS);
    assert_eq!(write.as_secs(), super::DEFAULT_COMMAND_IDLE_TIMEOUT_SECS);
}

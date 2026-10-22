use super::*;

#[test]
fn a_writer_that_finishes_moves_the_generation() {
    let _g = gate();
    let dir = tmp("generation");
    let before = observe(&dir);

    {
        let _held = DataLock::acquire(&dir, "writer", Duration::from_secs(1)).unwrap();
        let during = observe(&dir);
        assert_eq!(
            during.count, before.count,
            "the count must not move while the writer is still going"
        );
        assert!(
            during.writer_active,
            "and the reader must be able to see it"
        );
        assert!(
            !before.spans_one_moment(during),
            "a read that started before this writer and ended inside it saw two moments"
        );
    }

    let after = observe(&dir);
    assert_eq!(after.count, before.count.map(|c| c.wrapping_add(1)));
    assert!(!after.writer_active);
    assert!(
        !before.spans_one_moment(after),
        "a writer committed in between, so the reader must read again"
    );
    assert!(
        after.spans_one_moment(observe(&dir)),
        "and a quiet moment compares equal to itself"
    );
}

use super::*;

/// **A counter that cannot be read is unknown, and only a *missing* one is zero.** The
/// sibling test covers a file whose contents do not parse; both of its cases reach `observe`
/// through `Ok`, so nothing exercised the `NotFound` guard and reading it as `true` â€” every
/// IO error becoming generation zero â€” survived. A directory is not readable as a file on
/// any platform this ships to, and the error it raises is never `NotFound`.
#[test]
fn a_generation_file_that_cannot_be_read_is_unknown_not_zero() {
    let dir = tmp("generation-unreadable");
    std::fs::create_dir_all(dir.join(GENERATION_FILE)).unwrap();
    assert_eq!(
        observe(&dir).count,
        None,
        "an unreadable counter read as generation zero, which is LOWER than any real one, so two observations straddling a writer compare equal and a torn read is called one moment"
    );
}

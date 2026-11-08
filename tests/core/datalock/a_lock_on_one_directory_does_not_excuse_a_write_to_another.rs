use super::*;

/// **A lock on one data directory is not a lock on another.**
///
/// The self-hold question used to be one process-wide count, so a run holding the lock on its
/// own directory made every later `for_this_write` skip the lock -- including a write into a
/// directory nothing had locked, which is the race the lock exists to prevent. The fact is
/// per directory, which is what the lock covers.
#[test]
fn a_lock_on_one_directory_does_not_excuse_a_write_to_another() {
    let _g = gate();
    let held_dir = tmp("held-directory");
    let written_dir = tmp("written-directory");
    let _data_dir = DataDir::at(&written_dir);

    let outer = DataLock::acquire(&held_dir, "outer", Duration::from_secs(1)).unwrap();
    assert!(held_on(&held_dir));
    assert!(
        !held_on(&written_dir),
        "a lock on one directory was read as a lock on another"
    );

    let taken = DataLock::for_this_write("a ledger").unwrap();
    assert!(
        taken.is_some(),
        "a lock held on another directory excused this write from taking the one it writes under"
    );
    assert!(
        lock_file_is_in(&written_dir),
        "the write locked somewhere other than the directory it was pointed at"
    );

    drop(taken);
    drop(outer);
}

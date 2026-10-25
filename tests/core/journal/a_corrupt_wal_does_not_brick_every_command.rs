use super::*;

#[test]
fn a_corrupt_wal_does_not_brick_every_command() {
    // S10: a bad parse used to fail App::new and therefore every command. It must
    // instead recover: move the bad file aside, start fresh, and still construct.
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("journal.json");
    std::fs::write(&path, b"{ this is not valid json ]]").unwrap();

    let journal = Journal::at(path.clone()).expect("a corrupt WAL must not fail construction");

    // Started fresh...
    assert!(!journal.needs_recovery());
    // ...the bad file was set aside for inspection...
    let backup = {
        let mut s = path.clone().into_os_string();
        s.push(".corrupt");
        std::path::PathBuf::from(s)
    };
    assert!(
        backup.exists(),
        "the corrupt WAL should be preserved at {:?}",
        backup
    );
    // ...and it is no longer at the live path (so it won't re-trigger).
    assert!(
        !path.exists(),
        "the corrupt WAL should have been moved off the live path"
    );
}

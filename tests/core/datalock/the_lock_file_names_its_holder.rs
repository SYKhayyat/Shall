use super::*;

#[test]
fn the_lock_file_names_its_holder() {
    let _g = gate();
    let dir = tmp("holder");
    let _held = DataLock::acquire(&dir, "sync", Duration::from_secs(1)).unwrap();
    let stamp = DataLock::holder(&dir.join("shall.lock.owner"));
    assert!(stamp.contains("sync"), "{}", stamp);
    assert!(
        stamp.contains(&std::process::id().to_string()),
        "a holder nobody can identify is the message this exists to avoid: {}",
        stamp
    );
}

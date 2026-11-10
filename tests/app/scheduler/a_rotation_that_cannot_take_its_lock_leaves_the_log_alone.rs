use super::*;

/// **Rotation is skipped, not queued, when another process is already rotating.**
///
/// Every Shall process passes this code from `main`, before the data lock, because every one of
/// them appends to `schedule.log` under systemd's `append:` or launchd's `StandardOutPath`.
/// The lock is what makes a rotation one action: without it two processes both saw a log over
/// the cap, and the second one's `remove_file` deleted the generation the first had just set
/// aside.
#[test]
fn a_rotation_that_cannot_take_its_lock_leaves_the_log_alone() {
    let _env = shall::core::shall_data_dir_lock();
    let dir = std::env::temp_dir().join(format!("shall-rot-lock-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let previous = std::env::var_os("SHALL_DATA_DIR");
    std::env::set_var("SHALL_DATA_DIR", &dir);

    let log = dir.join("schedule.log");
    std::fs::write(&log, vec![b'x'; 11 * 1024 * 1024]).unwrap();
    std::fs::write(
        dir.join("schedule.log.1"),
        b"the generation already set aside",
    )
    .unwrap();

    // Somebody else's rotation, in progress.
    let held = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("schedule.log.lock"))
        .unwrap();
    fs2::FileExt::lock_exclusive(&held).unwrap();

    rotate_log_if_large(false);

    assert!(
        log.exists(),
        "a rotation ran without the lock, which is how one process deletes another's work"
    );
    assert_eq!(
        std::fs::read(dir.join("schedule.log.1")).unwrap(),
        b"the generation already set aside",
        "the previous generation was replaced by a rotation that did not hold the lock"
    );

    fs2::FileExt::unlock(&held).unwrap();
    // Control: the same call, with the lock free, does rotate — so the assertion above is
    // about the lock and not about a rotation that never worked.
    rotate_log_if_large(false);
    assert!(!log.exists(), "the control rotation did not happen");
    assert_eq!(
        std::fs::read(dir.join("schedule.log.1")).unwrap().len(),
        11 * 1024 * 1024,
        "the control rotation did not replace the old generation"
    );

    match previous {
        Some(v) => std::env::set_var("SHALL_DATA_DIR", v),
        None => std::env::remove_var("SHALL_DATA_DIR"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

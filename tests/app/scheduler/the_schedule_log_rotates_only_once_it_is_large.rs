use super::*;

/// The rotation is a size floor, not a policy debate: under it nothing happens (and a
/// missing log is not an error), over it the file becomes `.log.1` and any previous `.1`
/// is gone. Runs under the suite-wide data-dir lock because it writes beside the real
/// data directory.
#[test]
fn the_schedule_log_rotates_only_once_it_is_large() {
    let _env = shall::core::shall_data_dir_lock();
    let dir = std::env::temp_dir().join(format!("shall-rot-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let previous = std::env::var_os("SHALL_DATA_DIR");
    std::env::set_var("SHALL_DATA_DIR", &dir);

    // Small file: left alone.
    let log = dir.join("schedule.log");
    std::fs::write(&log, vec![b'x'; 1024]).unwrap();
    rotate_log_if_large();
    assert!(log.exists());
    assert!(!dir.join("schedule.log.1").exists());

    // Over the cap: becomes .1, and a pre-existing .1 is replaced.
    std::fs::write(dir.join("schedule.log.1"), b"old generation").unwrap();
    std::fs::write(&log, vec![b'x'; 11 * 1024 * 1024]).unwrap();
    rotate_log_if_large();
    assert!(!log.exists(), "the large log should have been renamed away");
    let rotated = std::fs::read(dir.join("schedule.log.1")).unwrap();
    assert_eq!(rotated.len(), 11 * 1024 * 1024, "the new generation wins");

    match previous {
        Some(v) => std::env::set_var("SHALL_DATA_DIR", v),
        None => std::env::remove_var("SHALL_DATA_DIR"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

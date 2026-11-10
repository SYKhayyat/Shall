use super::*;

/// The reported failure, run for real: two processes that both see an 11 MiB log rotate it at
/// the same moment.
///
/// The old code did `remove_file(schedule.log.1)` then `rename(schedule.log, schedule.log.1)`.
/// With both processes over the cap at once, the loser's `remove_file` ran after the winner's
/// rename — **deleting the generation the winner had just set aside** — and its own rename then
/// failed with ENOENT, printing `could not rotate schedule.log` on whatever command it was.
///
/// Five rounds of two threads, because the interleaving is the point: one round can pass by
/// luck. The content is the assertion, not the count — whatever happens, an 11 MiB log must
/// have become the `.1` generation.
#[test]
fn two_rotations_of_one_log_never_lose_it() {
    let _env = shall::core::shall_data_dir_lock();
    let dir = std::env::temp_dir().join(format!("shall-rot-race-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let previous = std::env::var_os("SHALL_DATA_DIR");
    std::env::set_var("SHALL_DATA_DIR", &dir);

    let log = dir.join("schedule.log");
    let rotated = dir.join("schedule.log.1");
    for round in 0..5 {
        let _ = std::fs::remove_file(&rotated);
        std::fs::write(&log, vec![b'x'; 11 * 1024 * 1024]).unwrap();

        let racers: Vec<std::thread::JoinHandle<()>> = (0..2)
            .map(|_| std::thread::spawn(|| rotate_log_if_large(false)))
            .collect();
        for racer in racers {
            racer.join().expect("a rotation thread panicked");
        }

        assert!(
            rotated.exists(),
            "round {round}: the rotated log is gone — a second rotation deleted the generation \
             the first had just set aside"
        );
        assert_eq!(
            std::fs::metadata(&rotated).unwrap().len(),
            11 * 1024 * 1024,
            "round {round}: the generation that survived is not the log that was rotated"
        );
    }

    match previous {
        Some(v) => std::env::set_var("SHALL_DATA_DIR", v),
        None => std::env::remove_var("SHALL_DATA_DIR"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

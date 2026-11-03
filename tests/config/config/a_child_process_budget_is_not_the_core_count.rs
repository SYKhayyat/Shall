use super::*;

/// **A child Shall only waits on is not a core** (`#111`, `C9`).
///
/// `max_parallel` defaults to `available_parallelism()`, and that is the right budget for work
/// that ends in a CPU. A manager being asked a question (`apt list`, `npm prefix -g`,
/// `brew info`) is a spawn and a wait, and on the two-core CI runners the fan-out was two
/// children wide however many managers the command had declared: seven children summing to
/// **5.23 s inside 4.25 s** of wall clock, four waves, two slots.
///
/// The default is 16 for `network_parallel`'s reason — the cost of one more wait is a file
/// descriptor, not a core — and it is a **default, not the core count**, because the machines
/// that pay most for the wrong number are the ones with the fewest cores.
#[test]
fn the_child_process_budget_defaults_to_sixteen_whatever_the_core_count() {
    let config = Config::default();
    assert_eq!(
        config.spawn_parallel, 16,
        "the budget for children Shall only waits on must not follow the core count"
    );

    // The control: the CPU budget still does, because work that ends in a CPU is work a core
    // does. If this ever moves, the two numbers have become one and #111 is back.
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    assert_eq!(
        config.max_parallel, cores,
        "`max_parallel` is the CPU budget and still follows the machine"
    );
}

/// A setting, not a constant: a preferences file can move it.
#[test]
fn a_child_process_budget_in_the_file_is_the_budget_the_run_uses() {
    let (dir, path) = temp_preferences("spawn-three", "spawn_parallel = 3\n");
    let config = Config::from_file(&path).expect("three is a budget");
    assert_eq!(config.spawn_parallel, 3);
    let _ = std::fs::remove_dir_all(&dir);
}

/// **Zero is not a budget**, and refusing it is not pedantry: `buffered(0)` opens nothing at
/// all, and the `.max(1)` clamps around every fan-out would have silently turned the setting
/// into one-they-think-is-sixteen. The three limits are refused in one sentence so the reader
/// sees them together.
#[test]
fn a_child_process_budget_of_zero_is_refused() {
    let (dir, path) = temp_preferences("spawn-zero", "spawn_parallel = 0\n");
    let err = Config::from_file(&path).expect_err("a fan-out with no width is not a setting");
    let text = err.to_string();
    assert!(
        text.contains("spawn_parallel"),
        "the refusal has to name the key the user wrote: {text}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// One file, one directory, one test at a time: the suite runs these in parallel and two tests
/// sharing a path would read each other's bytes.
fn temp_preferences(name: &str, body: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("shall-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("preferences.toml");
    std::fs::write(&path, body).expect("temp preferences");
    (dir, path)
}

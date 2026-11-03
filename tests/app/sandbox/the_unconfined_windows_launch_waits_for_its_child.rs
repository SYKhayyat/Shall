use super::*;

/// The unconfined Windows launch **waits** for the child it starts, and the whole argv is
/// asserted rather than the one word (#133).
///
/// `start` without `/wait` returns as soon as the process has been created: `cmd /c` then
/// exits and `status()` reports `start`'s status — almost always 0 — before the wrapped
/// command has done anything. The child is left outside the process tree, so its exit code is
/// lost, its output races Shall's own exit line, and the guard's kill-the-tree path does not
/// reach it. Everything a caller of `shall run` can observe is wrong at once, which is why the
/// argv is compared exactly: the title slot, the priority class, the command and its arguments
/// are one line, and a test that looks for a single word passes on a line that runs nothing.
///
/// Written as data rather than as a spawned process because no host in the local chain can
/// execute `cfg(windows)`: `cargo check --target x86_64-pc-windows-gnu` type-checks the block
/// and runs none of it, which is how the missing flag survived.
#[test]
fn the_unconfined_windows_launch_waits_for_its_child() {
    let args = vec!["--version".to_string(), "a b".to_string()];

    assert_eq!(
        Sandbox::windows_unconfined_argv("cargo", &args),
        vec![
            "/c".to_string(),
            "start".to_string(),
            // The window-title slot: without it `start` reads the first QUOTED argument as
            // the title and runs nothing at all.
            String::new(),
            "/low".to_string(),
            // The flag that makes the launch synchronous — the shape `shall run` is.
            "/wait".to_string(),
            "/b".to_string(),
            "cargo".to_string(),
            "--version".to_string(),
            "a b".to_string(),
        ]
    );
}

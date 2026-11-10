#![cfg(unix)]

use super::*;
use std::time::Duration;

/// The shim check is asked about **files this program does not own** — every `PATH` candidate
/// for every `shall run`, every entry in a shared `bin_dir`. It used to read each one whole
/// before deciding, which is not only expensive: a `PATH` entry that is not a regular file — a
/// fifo, a device, a socket — has no end to read to, so the read blocks and `shall run` hangs
/// before it has spawned anything.
///
/// A fifo is the cheapest way to make that failure deterministic: no writer, so `fs::read`
/// never returns.
#[tokio::test]
async fn a_path_entry_that_is_not_a_file_does_not_stall_the_search() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("usr-bin");
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let fifo = dir.join(deployed_name("jq"));

    let made = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    assert!(made, "this fixture needs `mkfifo` on the host");

    let path = std::env::join_paths([&dir]).unwrap();
    let found = tokio::time::timeout(Duration::from_secs(10), real_program_on("jq", Some(path)))
        .await
        .expect("the shim check read a fifo and never returned");

    // Not a shim, so the answer is the candidate the OS would have tried — the point is only
    // that asking cost a `stat` rather than an unbounded read.
    assert_eq!(found, fifo.to_string_lossy());
}

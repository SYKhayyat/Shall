use super::*;

/// **A removal that leaves a name answering is not a removal** (`#136`).
///
/// `nimble uninstall -y nimjson` returns 0 and drops the package from its own listing, and
/// `nimjson.cmd` stays in `~/.nimble/bin`, which is on PATH — measured on the Windows sweep
/// (`42361477256`, job `128360605190`). Shall deletes no files a manager wrote, so what it owes
/// the user is the sentence, and the sentence has to name the file or it is not actionable.
///
/// What this pins is the *rule for deciding what was left*, which is the part a deletion would
/// have had to get right too: exact name or a dotted extension, never a prefix. A manager that
/// removes `pv` must not be accused of leaving `pvx.cmd`, and a launcher with no extension
/// (`nimjson`) is the same leftover as one with.
#[test]
fn a_leftover_launcher_is_named_and_a_prefix_sibling_is_not() {
    // A directory this test owns: `SANDBOX` is a path config values are *written into text*
    // with, not one a test may create, and it is not writable on a normal host.
    let dir = std::env::temp_dir().join(format!("shall-leftover-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for file in [
        "nimjson",
        "nimjson.cmd",
        "nimjson.exe",
        "nimjsonx.cmd",
        "elsewhere.txt",
    ] {
        std::fs::write(dir.join(file), b"#!/bin/sh\n").unwrap();
    }

    let mut found: Vec<String> = leftover_entries(std::slice::from_ref(&dir), "nimjson")
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec!["nimjson", "nimjson.cmd", "nimjson.exe"],
        "a leftover is the name itself or a dotted extension of it, and nothing else"
    );

    // A manager with no bin directory has left nothing there — a missing directory is not a
    // permission problem and must not become a warning about a file nobody can point at.
    assert!(leftover_entries(&[dir.join("absent")], "nimjson").is_empty());
    // And a directory that cannot be read is skipped rather than fatal.
    assert!(leftover_entries(&[dir.join("absent").join("deeper")], "nimjson").is_empty());
}

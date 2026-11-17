//! An orphan schedule this shell cannot remove must not fail the sync (#173).
//!
//! `remove_what_is_no_longer_declared` (`#158`) is the only phase that starts from the *machine*
//! rather than from a line: it lists what the scheduler holds and takes off anything the files no
//! longer declare. Every removal in it was `?`, so the first one the OS refused ended the run —
//! and the refusal was over something the run had **not declared and could not remove**.
//!
//! Measured on this repository's Windows host: Task Scheduler answers `Access is denied.` to a
//! filtered token, so one stale task made *every* `sync` on that machine exit 1 while every line
//! in the files was already in effect. Sixteen end-to-end tests failed on one foreign task.
//!
//! **`extras::reconcile` already answers this the other way, on the same resources.** Its doc:
//! *"a backend that cannot undo one extra must not block the rest, so each failure warns and the
//! run continues."* The two functions remove schedules by the same means; only one of them said
//! so. The fix is that rule, and the two facts it must keep: the run **succeeds**, and the
//! schedule that is still firing is **named** rather than dropped.
//!
//! **Linux only, and it said "every Unix" until an aarch64 Mac ran it.** The unit is a file in the
//! fixture's `~/.config/systemd/user`, which is how the systemd provisioner's `installed()` finds
//! it; macOS's provisioner lists `~/Library/LaunchAgents/*.plist` and would find nothing here, so
//! the sweep would have no orphan to fail on and the assertion below would fail for the fixture's
//! reason rather than the product's. The refusal itself is hermetic — a stub says the unit is
//! still `active` after removal — but the *fixture* is systemd's, and a Mac needs a plist.
//!
//! The Windows half of `#173` is `tests/app/scheduler/a_refused_removal_says_what_to_do_not_what_is_wrong.rs`,
//! which is where the refusal it was measured on actually happens (a filtered token, elevation).
//! The behaviour this test guards — one unremovable orphan does not fail the run, and it is named
//! — is platform-independent, and `remove_what_is_no_longer_declared` gives it to every platform.

#[cfg(target_os = "linux")]
use crate::harness::Fixture;

/// A `systemctl` that says nothing and refuses nothing, and an `is-active` that always answers
/// `active` — so a unit Shall is told to remove is still running afterwards, which is exactly the
/// end state `remove_task` reports as a failure.
#[cfg(target_os = "linux")]
fn systemctl_that_never_stops() -> &'static str {
    "#!/bin/sh\n\
     case \"$*\" in\n\
     \x20 *is-active*) echo active; exit 0 ;;\n\
     esac\n\
     exit 0\n"
}

#[cfg(target_os = "linux")]
#[test]
fn an_orphan_that_cannot_be_removed_does_not_fail_the_sync() {
    let f = Fixture::new("173-orphan-sweep");
    // A schedule Shall wrote and the files do not declare. Written straight into the unit
    // directory, because that is the only place `installed()` looks and because a unit that
    // outlived its declaration is precisely what this phase exists to find.
    let units = f.root.join(".config").join("systemd").join("user");
    std::fs::create_dir_all(&units).unwrap();
    std::fs::write(units.join("shall-ghost.timer"), "[Timer]\n").unwrap();
    std::fs::write(units.join("shall-ghost.service"), "[Service]\n").unwrap();

    // And one declaration of a resource that costs nothing, so the run has a phase to converge
    // and is not short-circuited as "nothing to do".
    let dir = f.root.join("wanted");
    f.write_module(&format!("dir:{}\n", crate::harness::decl(&dir)));

    let bin = f.root.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    for program in ["systemctl", "loginctl", "journalctl"] {
        let path = bin.join(program);
        std::fs::write(&path, systemctl_that_never_stops()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    let (out, code) = f.run_with_path(&bin, &["-y", "sync"]);

    assert_eq!(
        code, 0,
        "a schedule the files do not declare, which this shell cannot remove, failed the whole \
         sync — the state this issue is about:\n{}",
        out
    );
    assert!(
        dir.is_dir(),
        "the declared resource did not converge, so the run stopped before it:\n{}",
        out
    );
    assert!(
        out.contains("ghost"),
        "the schedule that is still firing was not named, so a converge that could not finish \
         its cleanup reads as one that did:\n{}",
        out
    );
    // **The removal got as far as the files and no further**, which is why this is the shape a
    // `?` could not survive: `remove_task` deletes the unit and *then* asks whether it is still
    // running. So the failure arrives after real work has been done, on an orphan the files do
    // not mention — and the run used to die there.
    assert!(
        !units.join("shall-ghost.timer").exists(),
        "the sweep did not run at all, so this test measured nothing:\n{}",
        out
    );
    assert!(
        out.contains("could not be removed"),
        "the removal failed and the run said nothing about it, which turns a schedule that still \
         fires into one nobody knows about:\n{}",
        out
    );
}

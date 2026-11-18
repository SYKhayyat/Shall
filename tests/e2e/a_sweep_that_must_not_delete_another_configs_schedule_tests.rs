//! **The sweep takes off what *this run* wrote, and nothing else** (#182).
//!
//! `remove_what_is_no_longer_declared` (`#158`) is the one phase that starts from the machine: it
//! lists every `shall-` task and removes what the files no longer declare. On a machine with two
//! configs, that used to delete *the other config's* schedules — nothing recorded which config
//! declared a schedule, so `sync` in one removed every `shall-` task the files did not name,
//! whatever had declared it.
//!
//! The command line each task would fire now says whose it is (`schedule_origin`):
//!
//! - a task that names roots this run did not use was declared by another config — **left in
//!   place, and the run says so**;
//! - a task with no roots written before `c7dd0b67` is judged by its binary — **removed only
//!   when that binary is this one**;
//! - a task whose roots and binary are this run's is what the sweep has always removed.
//!
//! The classification itself lives below at
//! `tests/app/scheduler/the_origin_of_a_held_schedule_is_read_from_its_fire_line.rs`; these tests
//! drive the real sweep through a real binary with real unit files, because the defect was a
//! *sweep* that overreached, and only the sweep's own end-to-end passes or fails it.
//!
//! **Linux only**, like its sibling
//! `tests/e2e/an_orphan_that_cannot_be_removed_does_not_fail_the_sync_tests.rs`: the fixture's unit
//! directory is systemd's, and a Mac would scan `LaunchAgents` for nothing.

#[cfg(target_os = "linux")]
use crate::harness::Fixture;

#[cfg(target_os = "linux")]
fn systemctl_that_removes_quietly() -> &'static str {
    "#!/bin/sh\n\
     case \"$*\" in\n\
     \x20 *is-active*) echo inactive; exit 0 ;;\n\
     esac\n\
     exit 0\n"
}

#[cfg(target_os = "linux")]
fn seed_units(f: &Fixture) -> std::path::PathBuf {
    let units = f.root.join(".config").join("systemd").join("user");
    std::fs::create_dir_all(&units).unwrap();
    units
}

/// A task another config registered — its `--config-dir` is not this run's — must survive this
/// run's sweep, and the run must name the fact. This is the exact shape of the reported defect:
/// `sync` in config A deleting the schedules `sync` in config B had declared.
#[cfg(target_os = "linux")]
#[test]
fn a_task_that_names_another_configs_roots_survives_this_sync() {
    let f = Fixture::new("182-cross-config-sweep");
    let units = seed_units(&f);
    std::fs::write(
        units.join("shall-nightly.service"),
        "ExecStart=/srv/other/bin/shall --config-dir /srv/other/config --data-dir /srv/other/data sync\n",
    )
    .unwrap();
    std::fs::write(
        units.join("shall-nightly.timer"),
        "[Timer]\nOnCalendar=*-*-* 03:00:00\n",
    )
    .unwrap();

    // One declaration of a resource that costs nothing, so the run has a phase to converge and
    // is not short-circuited as "nothing to do" (the shape `#173` measured).
    let dir = f.root.join("wanted");
    f.write_module(&format!("dir:{}\n", crate::harness::decl(&dir)));

    let bin = f.root.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("systemctl"), systemctl_that_removes_quietly()).unwrap();
    std::fs::write(bin.join("loginctl"), "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::write(bin.join("journalctl"), "#!/bin/sh\nexit 0\n").unwrap();
    for program in ["systemctl", "loginctl", "journalctl"] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join(program), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }

    let (out, code) = f.run_with_path(&bin, &["-y", "sync"]);

    assert_eq!(
        code, 0,
        "another config's schedule failed this sync, which is the defect itself:\n{}",
        out
    );
    assert!(
        units.join("shall-nightly.service").exists(),
        "the sweep removed a task that names another config's roots — the defect:\n{}",
        out
    );
    assert!(
        units.join("shall-nightly.timer").exists(),
        "the sweep removed the timer of a task that names another config's roots:\n{}",
        out
    );
    assert!(
        out.contains("nightly") && out.contains("left it in place"),
        "the run did not name the task it left and why, so a sweep that silently skips reads \
         as one that looked:\n{}",
        out
    );
}

/// A task written before `c7dd0b67` records no roots; its binary is all it has. This run's sync
/// must still retire it when the binary is this very binary — `#158` cannot stop working for
/// upgraders, which is the C34 ruling's side — by removing it, exactly as the sweep always did.
#[cfg(target_os = "linux")]
#[test]
fn a_roots_less_task_by_this_binary_is_still_the_sweeps_business() {
    let f = Fixture::new("182-legacy-own-binary");
    let units = seed_units(&f);
    // The binary a firing would exec is the binary running this sync: no flags, written before
    // `c7dd0b67`, and there is exactly one thing that says whose it is.
    std::fs::write(
        units.join("shall-tidy.service"),
        format!("ExecStart={} sync\n", env!("CARGO_BIN_EXE_shall")),
    )
    .unwrap();
    std::fs::write(
        units.join("shall-tidy.timer"),
        "[Timer]\nOnCalendar=*-*-* 02:00:00\n",
    )
    .unwrap();

    let dir = f.root.join("wanted");
    f.write_module(&format!("dir:{}\n", crate::harness::decl(&dir)));

    let bin = f.root.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("systemctl"), systemctl_that_removes_quietly()).unwrap();
    std::fs::write(bin.join("loginctl"), "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::write(bin.join("journalctl"), "#!/bin/sh\nexit 0\n").unwrap();
    for program in ["systemctl", "loginctl", "journalctl"] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join(program), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }

    let (out, code) = f.run_with_path(&bin, &["-y", "sync"]);

    assert_eq!(
        code, 0,
        "retiring a roots-less task by this binary failed the sync:\n{}",
        out
    );
    assert!(
        !units.join("shall-tidy.service").exists(),
        "the sweep did not remove a roots-less task whose binary is this binary — #158 \
         stopped working for upgraders:\n{}",
        out
    );
    assert!(
        out.contains("removed 1 schedule(s)"),
        "the removal was not reported:\n{}",
        out
    );
}

/// And the same roots-less task whose binary is *another* Shall must survive, whatever its
/// name's prefix suggests: the binary is the only authorship a legacy task has.
#[cfg(target_os = "linux")]
#[test]
fn a_roots_less_task_by_another_binary_is_left_in_place() {
    let f = Fixture::new("182-legacy-foreign-binary");
    let units = seed_units(&f);
    std::fs::write(
        units.join("shall-tidy.service"),
        format!("ExecStart={} sync\n", "/usr/local/other/bin/shall"),
    )
    .unwrap();
    std::fs::write(
        units.join("shall-tidy.timer"),
        "[Timer]\nOnCalendar=*-*-* 02:00:00\n",
    )
    .unwrap();

    let dir = f.root.join("wanted");
    f.write_module(&format!("dir:{}\n", crate::harness::decl(&dir)));

    let bin = f.root.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("systemctl"), systemctl_that_removes_quietly()).unwrap();
    std::fs::write(bin.join("loginctl"), "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::write(bin.join("journalctl"), "#!/bin/sh\nexit 0\n").unwrap();
    for program in ["systemctl", "loginctl", "journalctl"] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join(program), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }

    let (out, code) = f.run_with_path(&bin, &["-y", "sync"]);

    assert_eq!(
        code, 0,
        "another binary's legacy task failed this sync:\n{}",
        out
    );
    assert!(
        units.join("shall-tidy.service").exists(),
        "the sweep removed a legacy task that would fire a different binary:\n{}",
        out
    );
    assert!(
        out.contains("tidy") && out.contains("a different Shall binary"),
        "the run did not name the foreign task it left and why:\n{}",
        out
    );
}

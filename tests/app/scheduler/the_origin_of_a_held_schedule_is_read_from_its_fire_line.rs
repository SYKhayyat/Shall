//! **Whose a held schedule is comes out of the line it would fire** (#182).
//!
//! The sweep that removes schedules the files no longer declare used to take off *every*
//! `shall-`-prefixed task, so on a machine with two configs, `sync` in one deleted the other's
//! schedules. Every task registered since `c7dd0b67` carries the `--config-dir`/`--data-dir`
//! roots it was declared from, and a task older than that has exactly one surviving token of its
//! author — the binary in its argv. `schedule_origin` turns that line into one of four answers,
//! and the sweep may remove **only** `Ours`.
//!
//! These are the classification's own tests; the sweep that consumes them is pinned at
//! `tests/e2e/a_sweep_that_must_not_delete_another_configs_schedule_tests.rs`.

use super::*;

/// The run these tests stand in for: a config at `/srv/app/config` and data at `/srv/app/data`.
fn app_roots() -> Roots {
    Roots {
        config: PathBuf::from("/srv/app/config"),
        data: PathBuf::from("/srv/app/data"),
    }
}

/// The binary the tests stand in for — and the one a legacy task written by the same checkout
/// would carry.
fn app_bin() -> PathBuf {
    PathBuf::from("/srv/app/bin/shall")
}

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

/// The task this run declared: both roots written, because both were non-default.
#[test]
fn a_task_whose_roots_are_this_runs_is_ours() {
    let line = argv(&[
        "/srv/app/bin/shall",
        "--config-dir",
        "/srv/app/config",
        "--data-dir",
        "/srv/app/data",
        "sync",
    ]);
    assert_eq!(
        schedule_origin(&line, false, &app_roots(), &app_bin()),
        ScheduleOrigin::Ours
    );
}

/// Another config's root in the line is the whole defect: this run must not retire it.
#[test]
fn a_task_that_names_another_config_is_not_ours_even_for_this_binary() {
    let line = argv(&[
        "/srv/app/bin/shall",
        "--config-dir",
        "/srv/other/config",
        "--data-dir",
        "/srv/app/data",
        "sync",
    ]);
    match schedule_origin(&line, false, &app_roots(), &app_bin()) {
        ScheduleOrigin::AnotherConfig { config, data } => {
            assert_eq!(config.as_deref(), Some("/srv/other/config"));
            assert_eq!(data.as_deref(), Some("/srv/app/data"));
        }
        other => panic!("a configuring root that differs was read as {other:?}"),
    }
}

/// A root with no flag means the declaring run used its default — so a task that forgot the
/// data flag is only ours if this run also used the default data root, however equal the config
/// root is.
#[test]
fn a_missing_flag_is_a_default_root_and_not_our_non_default_one() {
    // Declared from a run whose data root was the default; this run's data is `/srv/app/data`.
    let line = argv(&[
        "/srv/app/bin/shall",
        "--config-dir",
        "/srv/app/config",
        "sync",
    ]);
    assert_eq!(
        schedule_origin(&line, false, &app_roots(), &app_bin()),
        ScheduleOrigin::AnotherConfig {
            config: Some("/srv/app/config".to_string()),
            data: None,
        }
    );

    // The same line, judged by a run that also uses the default data root: ours.
    let same_data_default = Roots {
        config: PathBuf::from("/srv/app/config"),
        data: shall::utils::safe_data_dir(),
    };
    assert_eq!(
        schedule_origin(&line, false, &same_data_default, &app_bin()),
        ScheduleOrigin::Ours
    );
}

/// A task registered before `c7dd0b67` records no roots; its binary is the only authorship that
/// survives. The C34 ruling: sweep it when that binary is this one — and never otherwise.
#[test]
fn a_task_with_no_roots_is_ours_only_when_it_would_fire_this_binary() {
    let line = argv(&["/srv/app/bin/shall", "sync"]);
    assert_eq!(
        schedule_origin(&line, false, &app_roots(), &app_bin()),
        ScheduleOrigin::Ours
    );

    let foreign = argv(&["/usr/local/other/bin/shall", "sync"]);
    assert_eq!(
        schedule_origin(&foreign, false, &app_roots(), &app_bin()),
        ScheduleOrigin::ForeignBinary {
            binary: "/usr/local/other/bin/shall".to_string(),
        }
    );
}

/// The flags Shall writes sit immediately after the binary; a `--config-dir` inside the *command*
/// is the command's own business and identifies nothing.
#[test]
fn a_flag_inside_the_command_does_not_name_the_declaring_run() {
    let line = argv(&["/srv/app/bin/shall", "run", "--config-dir", "/srv/other"]);
    assert_eq!(
        schedule_origin(&line, false, &app_roots(), &app_bin()),
        ScheduleOrigin::Ours
    );
}

/// The quoting `Roots::flag` writes for a path with a space survives the split: the read-back
/// compares the unquoted value, exactly as the writer's consumers see it.
#[test]
fn a_quoted_root_value_reads_back_unquoted() {
    let roots = Roots {
        config: PathBuf::from("/srv/app config"),
        data: PathBuf::from("/srv/app data"),
    };
    let line = argv(&[
        "/srv/app/bin/shall",
        "--config-dir",
        "/srv/app config",
        "--data-dir",
        "/srv/app data",
        "sync",
    ]);
    assert_eq!(
        schedule_origin(&line, false, &roots, &app_bin()),
        ScheduleOrigin::Ours
    );
}

/// A flag without its value is a line no renderer of ours writes: not ours, on no one's word.
#[test]
fn a_dangling_flag_is_unreadable() {
    let line = argv(&["/srv/app/bin/shall", "--config-dir"]);
    assert_eq!(
        schedule_origin(&line, false, &app_roots(), &app_bin()),
        ScheduleOrigin::Unreadable
    );
}

/// No command line at all: a store that can certify authorship (systemd's unit file, launchd's
/// plist) answers `Ours` — the `#158` gate and the `#173` pin's shape; a store with nothing
/// left to ask does not get to call the task ours.
#[test]
fn an_unreadable_line_falls_back_to_the_stores_word() {
    assert_eq!(
        schedule_origin(&[], true, &app_roots(), &app_bin()),
        ScheduleOrigin::Ours
    );
    assert_eq!(
        schedule_origin(&[], false, &app_roots(), &app_bin()),
        ScheduleOrigin::Unreadable
    );
}

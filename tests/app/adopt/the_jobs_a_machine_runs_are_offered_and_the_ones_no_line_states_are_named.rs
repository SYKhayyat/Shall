//! C41, end to end: the jobs a machine runs arrive as `schedule:` lines, the ones no line can
//! state arrive by name with the field that stopped them, and `--user` leaves `/etc` alone.

use super::*;
use shall::core::executor::StdOutput;
type Answer = Result<StdOutput>;

fn cron_answer(text: &'static str) -> Answer {
    Ok(DryRunOutput {
        stdout: text.as_bytes().to_vec(),
        stderr: vec![],
    }
    .into())
}

async fn discover_with(scope: AdoptScope, answers: Vec<(&'static str, Answer)>) -> Discovery {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (command, answer) in answers {
        mock.set_response(command, answer);
    }
    let config = Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec![],
            ..Default::default()
        },
        ..Config::default()
    };
    let state = Arc::new(Mutex::new(StateRegistry::default()));
    let adopt = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        state,
        &config,
        CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new())),
    );
    adopt.discover_scoped(&scope).await.expect("discovery")
}

/// The manifest the person actually reads. The readers can be right and the section still never
/// be written, which is the failure this asserts against.
#[tokio::test]
async fn the_manifest_writes_the_schedule_and_names_what_it_could_not_state() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "crontab -l",
        cron_answer("30 2 * * * /usr/bin/backup --all\n"),
    );
    mock.set_response_that_must_not_be_used(
        "systemctl --all --no-pager list-timers",
        cron_answer(""),
    );
    let config = Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec![],
            ..Default::default()
        },
        ..Config::default()
    };
    let state = Arc::new(Mutex::new(StateRegistry::default()));
    let adopt = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        state,
        &config,
        CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new())),
    );
    let found = adopt
        .discover_scoped(&AdoptScope {
            backends: vec![],
            user_only: true,
            ..AdoptScope::default()
        })
        .await
        .expect("discovery");
    let text = adopt.render_manifest(&found, &HashMap::new());

    assert!(
        text.contains("# === The jobs this machine already runs on a schedule ==="),
        "the section is missing from:\n{}",
        text
    );
    assert!(
        text.contains("1 schedule(s) were written to the `schedules` file, not here"),
        "the section must say where the line went, and why not here:\n{}",
        text
    );
    assert!(
        text.contains("They were read from crontab -l (this account)"),
        "the header names the dialect it read:\n{}",
        text
    );
}

fn line_of(found: &Discovery, name: &str) -> String {
    let job = found
        .schedules
        .iter()
        .find(|job| job.name == name)
        .unwrap_or_else(|| {
            panic!(
                "`{}` was not offered; offered: {:?}",
                name,
                found.schedules.iter().map(|j| &j.name).collect::<Vec<_>>()
            )
        });
    let mut line = format!(
        "schedule:{}@cron={},run={}",
        job.name, job.cron, job.command
    );
    if let Some(user) = &job.user {
        line.push_str(&format!(",user={}", user));
    }
    line
}

#[tokio::test]
async fn a_crontab_job_is_a_line_that_runs_the_same_command_at_the_same_times() {
    let found = discover_with(
        AdoptScope {
            backends: vec![],
            ..AdoptScope::default()
        },
        vec![(
            "crontab -l",
            cron_answer("MAILTO=me\n30 2 * * * /usr/bin/backup --all\n"),
        )],
    )
    .await;

    assert_eq!(
        line_of(&found, "cron-user-2"),
        "schedule:cron-user-2@cron=30 2 * * *,run=/usr/bin/backup --all",
        "a user crontab has no user column, so the line carries none"
    );
}

#[tokio::test]
async fn a_timer_is_a_line_and_a_boot_timer_is_named_with_the_field_that_stopped_it() {
    let found = discover_with(
        AdoptScope {
            backends: vec![],
            ..AdoptScope::default()
        },
        vec![
            (
                "systemctl --all --no-pager list-timers",
                cron_answer(
                    "NEXT LEFT LAST PASSED UNIT ACTIVATES\n\
                     n/a  n/a  n/a  n/a    shall-nightly.timer  shall-nightly.service\n\
                     n/a  n/a  n/a  n/a    shall-onboot.timer   shall-onboot.service\n",
                ),
            ),
            (
                "systemctl --no-pager show -p OnCalendar -p TimersCalendar -p TimersMonotonic shall-nightly.timer",
                cron_answer("OnCalendar=*-*-* 03:00:00\n"),
            ),
            (
                "systemctl --no-pager show -p OnCalendar -p TimersCalendar -p TimersMonotonic shall-onboot.timer",
                cron_answer("OnBootSec=15min\n"),
            ),
        ],
    )
    .await;

    assert_eq!(
        line_of(&found, "timer-system-shall-nightly-timer"),
        "schedule:timer-system-shall-nightly-timer@cron=0 3 * * *,run=systemctl start shall-nightly"
    );

    let (name, reason) = found
        .skipped
        .iter()
        .map(|s| (s.package.name.clone(), s.reason.clone()))
        .find(|(name, _)| name == "timer-system-shall-onboot-timer")
        .unwrap_or_else(|| panic!("the boot timer must be named: {:?}", found.skipped));
    assert!(name.starts_with("timer-"));
    assert!(
        reason.contains("OnBootSec") && reason.contains("calendar"),
        "the refusal says which field it was: {}",
        reason
    );
}

#[tokio::test]
async fn user_only_leaves_the_system_stores_unread() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // A root timer holds a job and this account's crontab holds another. `--user` keeps the
    // account's own and must not even ask the system instance — asking is what would let a root
    // job in.
    mock.set_response("crontab -l", cron_answer("15 4 * * * /usr/bin/mine\n"));
    mock.set_response_that_must_not_be_used(
        "systemctl --all --no-pager list-timers",
        cron_answer("UNIT ACTIVATES\nroot-job.timer root-job.service\n"),
    );

    let config = Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec![],
            ..Default::default()
        },
        ..Config::default()
    };
    let state = Arc::new(Mutex::new(StateRegistry::default()));
    let adopt = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        state,
        &config,
        CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new())),
    );
    let found = adopt
        .discover_scoped(&AdoptScope {
            backends: vec![],
            user_only: true,
            ..AdoptScope::default()
        })
        .await
        .expect("discovery");

    let names: Vec<&String> = found.schedules.iter().map(|j| &j.name).collect();
    assert_eq!(
        names,
        vec!["cron-user-1"],
        "`--user` offers this account's job and no root one"
    );
}

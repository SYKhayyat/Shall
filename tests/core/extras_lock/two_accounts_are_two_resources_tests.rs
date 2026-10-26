use shall::config::grammar::{Options, ResourceKind, Statement};
use shall::core::extras_lock::{
    extra_key, qualify_removal, split_removal, ExtraKey, ExtrasLedger, ACCOUNT_SCOPED,
};
use shall::core::LockFile;
use std::collections::BTreeSet;

fn me() -> String {
    shall::core::account::Account::current()
        .expect("this process has an account")
        .name()
        .to_string()
}

fn key(wire: &str) -> ExtraKey {
    wire.parse().expect("a well-formed ledger row")
}

fn record(wire: &str) -> shall::core::extras_lock::ExtraRecord {
    shall::core::extras_lock::ExtraRecord::new(&key(wire))
}

fn statement(kind: ResourceKind, name: &str, user: Option<&str>) -> Statement {
    let mut options = Options::default();
    if let Some(user) = user {
        options.set("user", user);
    }
    match kind {
        ResourceKind::Shim => Statement::Shim(name.to_string(), options),
        ResourceKind::Setting => Statement::Setting(name.to_string(), options),
        ResourceKind::Service => Statement::Service(name.to_string(), options),
        ResourceKind::Schedule => Statement::Schedule(name.to_string(), options),
        other => panic!("{other} is not one of the four account-scoped kinds"),
    }
}

#[test]
fn every_account_scoped_kind_keys_two_users_as_two_rows() {
    for kind in ACCOUNT_SCOPED {
        for name in ["thing", "org.gnome.desktop/theme"] {
            let mine = extra_key(&statement(*kind, name, Some(&me())))
                .unwrap()
                .unwrap()
                .to_string();
            let theirs = extra_key(&statement(*kind, name, Some("someone-else")))
                .unwrap()
                .unwrap()
                .to_string();
            assert_ne!(mine, theirs, "{kind}: two accounts wrote one row");
            assert_eq!(
                mine.parse::<ExtraKey>().unwrap().account_suffix(),
                Some(me().as_str()),
                "{kind}: the account did not survive its own row"
            );
        }
    }
}

#[test]
fn deleting_one_accounts_declaration_leaves_the_others_row_alone() {
    for kind in ACCOUNT_SCOPED {
        let name = "thing";
        let mine = extra_key(&statement(*kind, name, Some(&me())))
            .unwrap()
            .unwrap()
            .to_string();
        let theirs = extra_key(&statement(*kind, name, Some("someone-else")))
            .unwrap()
            .unwrap()
            .to_string();

        let mut ledger = ExtrasLedger::new();
        ledger.record(vec![record(&mine), record(&theirs)]);
        let drift: Vec<String> = ledger
            .drift(&[key(&mine)].into_iter().collect())
            .iter()
            .map(|record| record.wire())
            .collect();
        assert_eq!(
            drift,
            vec![theirs],
            "{kind}: removing one account's declaration drifted {}",
            drift.len()
        );
    }
}

#[test]
fn the_teardown_name_carries_the_account_and_reads_back_through_one_codec() {
    for kind in ACCOUNT_SCOPED {
        let qualified = qualify_removal(*kind, "thing", Some("someone-else")).unwrap();
        assert!(
            qualified.ends_with("@user=someone-else"),
            "{kind}: {qualified}"
        );
        let (bare, user) = split_removal(*kind, &qualified).unwrap();
        assert_eq!(bare, "thing", "{kind}: the subject did not survive");
        assert_eq!(user.as_deref(), Some("someone-else"), "{kind}");

        let plain = qualify_removal(*kind, "thing", None).unwrap();
        assert_eq!(plain, "thing", "{kind}");
        let (bare, user) = split_removal(*kind, &plain).unwrap();
        assert_eq!(bare.as_str(), "thing");
        assert!(user.is_none(), "{kind}");
    }
}

#[test]
fn a_row_whose_account_cannot_be_read_is_refused_rather_than_guessed() {
    for bad in ["thing@user=", "thing@user=a:b", "@user=alice"] {
        let err = split_removal(ResourceKind::Shim, bad)
            .expect_err("a malformed account must be refused");
        let err = err.to_string();
        assert!(err.contains("cannot be read back"), "{err}");
        assert!(err.contains("choosing an account"), "{err}");
    }
}

#[test]
fn a_path_resolved_kind_is_not_given_an_account() {
    for kind in [ResourceKind::Link, ResourceKind::Dir] {
        let err = qualify_removal(kind, "/home/someone/.vimrc", Some("someone"))
            .expect_err("a path row is not an account row");
        assert!(err.to_string().contains("not keyed by account"), "{err}");
    }
}

#[test]
fn a_named_accounts_shim_targets_that_accounts_own_bin_directory() {
    let account = shall::core::account::Account::current().unwrap();
    let configured = std::path::Path::new("/nonexistent/shall-configured-bin");
    let (dir, owner) = shall::app::shim_manager::shim_target(configured, Some(account.name()))
        .expect("the account resolves");
    assert_eq!(dir, account.bin_dir().unwrap());
    assert!(
        dir.starts_with(account.home()),
        "{dir:?} is not in the account's home"
    );
    assert_eq!(owner.unwrap().name(), account.name());

    let (dir, owner) =
        shall::app::shim_manager::shim_target(configured, None).expect("the default resolves");
    assert_eq!(dir, configured);
    assert!(owner.is_none());
}

#[test]
fn a_named_accounts_schedule_resolves_its_own_paths() {
    use shall::app::scheduler::TaskTarget;

    let account = shall::core::account::Account::current().unwrap();
    let target = TaskTarget::resolve(Some(account.name())).unwrap();
    assert!(target.is_scoped(), "a resolved account is not scoped");
    assert_eq!(target.account().unwrap(), account);
    assert!(target
        .account()
        .unwrap()
        .systemd_user_dir()
        .unwrap()
        .starts_with(account.home()));
    assert!(target
        .account()
        .unwrap()
        .launch_agents_dir()
        .unwrap()
        .starts_with(account.home()));
    assert!(target
        .account()
        .unwrap()
        .data_dir()
        .unwrap()
        .starts_with(account.home()));
}

#[tokio::test]
async fn deprovisioning_a_schedule_takes_the_account_from_the_row() {
    use crate::mock_providers::{mock_task_key, MockTaskProvisioner};
    use shall::app::scheduler::{SchedulerManager, TaskTarget};
    use std::sync::Arc;

    let account = shall::core::account::Account::current().unwrap();
    let provisioner = Arc::new(MockTaskProvisioner::new());
    let mut held = provisioner.active_tasks.lock().await;
    held.insert(
        mock_task_key(
            "nightly",
            &TaskTarget::resolve(Some(account.name())).unwrap(),
        ),
        schedule_config("nightly"),
    );
    held.insert(
        mock_task_key("nightly", &TaskTarget::resolve(None).unwrap()),
        schedule_config("nightly"),
    );
    drop(held);

    let manager = SchedulerManager::with_provisioner(provisioner.clone())
        .expect("a mock provisioner is usable");
    let executor = shall::core::CommandExecutor::new(false, false);

    manager
        .deprovision(
            &executor,
            "nightly",
            Some(account.name()),
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await,
        )
        .await
        .unwrap();
    let held = provisioner.active_tasks.lock().await;
    assert!(
        !held.contains_key(&mock_task_key(
            "nightly",
            &TaskTarget::resolve(Some(account.name())).unwrap()
        )),
        "the named row survived its own removal"
    );
    assert!(
        held.contains_key(&mock_task_key(
            "nightly",
            &TaskTarget::resolve(None).unwrap()
        )),
        "removing the named account's schedule also removed the unscoped one"
    );
}

fn schedule_config(name: &str) -> shall::config::config::ScheduleConfig {
    shall::config::config::ScheduleConfig {
        name: name.to_string(),
        cron: "0 2 * * *".to_string(),
        command: "sync".to_string(),
        notification: None,
        enabled: None,
        persistent: None,
        jitter: None,
        elevated: None,
        user: None,
    }
}

#[test]
fn a_setting_records_scope_and_account_without_either_replacing_the_other() {
    let four: BTreeSet<String> = [
        Options::default(),
        {
            let mut o = Options::default();
            o.set("scope", "system");
            o
        },
        {
            let mut o = Options::default();
            o.set("user", "someone-else");
            o
        },
        {
            let mut o = Options::default();
            o.set("scope", "system");
            o.set("user", "someone-else");
            o
        },
    ]
    .into_iter()
    .map(|o| {
        extra_key(&Statement::Setting("org.gnome.x/theme".into(), o))
            .unwrap()
            .unwrap()
            .to_string()
    })
    .collect();
    assert_eq!(four.len(), 4, "{four:?}");

    let scoped: ExtraKey = four
        .iter()
        .find(|row| row.contains("@scope=system") && row.contains("@user="))
        .expect("the scoped, named row is one of the four")
        .parse()
        .expect("the row reads back");
    assert_eq!(scoped.account_suffix(), Some("someone-else"));
    assert_eq!(scoped.subject, "org.gnome.x/theme@scope=system");
}

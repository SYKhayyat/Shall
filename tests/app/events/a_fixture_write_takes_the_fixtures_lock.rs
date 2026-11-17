//! **A fixture's write goes to the fixture, and so does its lock** (#185).
//!
//! `approve_all` writes `locks/hooks.toml` under the config it was loaded from — that half was
//! never in doubt — but the *lock* it took was `DataLock::for_this_write`, which defaulted to
//! `safe_data_dir()`: the machine's store, whatever config the write belonged to. So an
//! in-process test contended with every other `shall` on the host, and the failure it produced
//! named the developer's real data directory:
//!
//! ```text
//! this process already holds the lock on C:\Users\Administrator\AppData\Roaming\shall
//! ```
//!
//! That is the shape `#183` fixed the consequences of; this is the shape that made the contention
//! possible. `Config::data_root` is the store a run writes — it is configurable, and
//! `Config::sandboxed` points it at a temp directory — so the lock belongs there, and
//! `LockFile::update` now takes it as an argument rather than assuming.

use crate::harness::Fixture;
use shall::app::events::EventHooks;
use shall::config::Config;
use shall::core::hook_lock::HookLedger;
use shall::core::LockFile;
use shall::model::event::Event;
use std::path::Path;

/// Every assertion here is about a **directory**, and the one that matters is the negative: the
/// machine's lock file must not be the one this write took.
#[test]
fn a_fixture_write_takes_the_fixtures_lock_and_not_the_machines() {
    let f = Fixture::new("hermetic-approve");
    let root = f.root.clone();
    write_hook(&root, Event::OnDrift);

    let machine = shall::utils::safe_data_dir();
    let machine_lock_was_there = machine.join("shall.lock").exists();

    let config = Config::sandboxed(&root);
    let hooks = EventHooks::load(&config);
    assert_eq!(
        hooks.approve_all().unwrap(),
        1,
        "the approval still happens"
    );

    // 1. The record is under the fixture — the claim the test in the issue made and this one
    //    keeps: an approval is written where the config says, not where the machine says.
    let ledger = HookLedger::path_in(&config.layout().locks_dir());
    assert!(
        ledger.starts_with(&root),
        "the approvals were written to {}, which is outside the fixture {}",
        ledger.display(),
        root.display()
    );
    assert!(
        HookLedger::load(&ledger).unwrap().len() == 1,
        "the approval did not land in the ledger the fixture owns"
    );

    // 2. **The lock was taken in the store being written.** `shall.lock` is created next to the
    //    lock it protects and stays there after the lock is released, so its existence is the
    //    record of which directory the write locked. Under the old `for_this_write` there was no
    //    such file here — the one it created was the machine's.
    assert!(
        root.join("shall.lock").exists(),
        "no lock was taken in the fixture's own store, so the lock it took was somewhere else"
    );

    // 3. And the machine's store was left alone. Only the "did not appear" direction is asserted:
    //    if the file was already there some other `shall` owns it, and this test cannot say what
    //    that process did while it ran.
    if !machine_lock_was_there {
        assert!(
            !machine.join("shall.lock").exists(),
            "this write created a lock in the machine's data directory ({}) — the same contention \
             with every other `shall` on the host that made this issue",
            machine.display()
        );
    }
}

/// **The product's own reason for the parameter, not just the fixture's.** `data_root` is a
/// setting: a user can point the store somewhere else than the platform default, and a lock taken
/// on the default would then protect a directory their writes never reach.
#[test]
fn a_configured_data_root_is_where_the_lock_is_taken() {
    let f = Fixture::new("configured-data-root");
    let root = f.root.clone();
    write_hook(&root, Event::OnDrift);

    // A data root nowhere near the config, which is what `data_root` in the settings is for.
    let store = root.join("elsewhere/state");
    std::fs::create_dir_all(&store).unwrap();
    let config = Config {
        data_root: store.clone(),
        ..Config::sandboxed(&root)
    };

    assert_eq!(EventHooks::load(&config).approve_all().unwrap(), 1);

    assert!(
        store.join("shall.lock").exists(),
        "the lock belongs to the configured data root ({}), and nothing was locked there",
        store.display()
    );
    assert!(
        !root.join("shall.lock").exists(),
        "the lock was taken on the config root as well, which is a second lock on a store this \
         write does not use"
    );
}

/// And a write with no lock root of its own still approves — the fix must not be "do nothing when
/// the store is unusual".
#[test]
fn an_empty_hook_set_approves_nothing_without_writing() {
    let f = Fixture::new("empty-approve");
    let config = Config::sandboxed(&f.root);
    assert_eq!(EventHooks::load(&config).approve_all().unwrap(), 0);
    assert!(
        !HookLedger::path_in(&config.layout().locks_dir()).exists(),
        "an empty approval set wrote a ledger anyway"
    );
}

fn write_hook(root: &Path, event: Event) {
    let dir = root.join("hooks");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(event.as_str()), "#!/bin/sh\ntrue\n").unwrap();
}

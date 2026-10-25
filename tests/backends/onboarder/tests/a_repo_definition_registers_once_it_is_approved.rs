use super::*;

/// 7a: the definition travels with the repo. A machine that has never seen this file
/// registers the backend from it — after `shall lock`, because the file is argv the repo
/// can run and that is II.12's question, not a new one.
#[test]
fn a_repo_definition_registers_once_it_is_approved() {
    use shall::core::hook_lock::{adapter_id, hash_script, HookLedger};
    use shall::core::LockFile;
    let tmp = tempfile::tempdir().unwrap();
    let (_, exec) = mock_exec();
    write_repo(tmp.path(), PARU_TOML);
    let locks = tmp.path().join("locks");

    // Unapproved: nothing registers, however valid the definition is.
    let mut reg = BackendRegistry::new();
    let n = load_custom_backends_from(&mut reg, &exec, &tmp.path().join("backends.toml"), &locks);
    assert_eq!(n, 0, "an unapproved definition file registered a backend");
    assert!(reg.get("paru").is_none());

    // What `shall lock` writes.
    let mut ledger = HookLedger::new();
    ledger.approve(&adapter_id("backends.toml"), &hash_script(PARU_TOML));
    ledger.save(&HookLedger::path_in(&locks)).unwrap();

    let mut reg = BackendRegistry::new();
    let n = load_custom_backends_from(&mut reg, &exec, &tmp.path().join("backends.toml"), &locks);
    assert_eq!(n, 1);
    assert!(reg.get("paru").is_some());
}

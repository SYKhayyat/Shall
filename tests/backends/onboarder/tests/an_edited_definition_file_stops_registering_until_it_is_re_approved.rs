use super::*;

/// And the case the ledger exists for: approved once, then edited. An added `[[backend]]`
/// is a new command the repo can run, so one identity covers the whole file.
#[test]
fn an_edited_definition_file_stops_registering_until_it_is_re_approved() {
    use shall::core::hook_lock::{adapter_id, hash_script, HookLedger};
    let tmp = tempfile::tempdir().unwrap();
    let (_, exec) = mock_exec();
    let locks = tmp.path().join("locks");
    let mut ledger = HookLedger::new();
    ledger.approve(&adapter_id("backends.toml"), &hash_script(PARU_TOML));
    ledger.save(&HookLedger::path_in(&locks)).unwrap();

    let edited = format!(
        "{}\n[[backend]]\nname = \"yay\"\ninstall_args = [\"-S\"]\n",
        PARU_TOML
    );
    write_repo(tmp.path(), &edited);

    let mut reg = BackendRegistry::new();
    let n = load_custom_backends_from(&mut reg, &exec, &tmp.path().join("backends.toml"), &locks);
    assert_eq!(n, 0, "an edited file kept running on the old approval");
    assert!(
        reg.get("paru").is_none(),
        "the unchanged half kept running too"
    );
}

use super::*;

/// U2: a custom backend gains a capability only when its definition provides the fields
/// for it — and gains it when it does. Absent stays absent (the safe default), present
/// makes it a first-class peer.
#[test]
fn a_custom_backend_is_a_first_class_peer_when_it_says_so() {
    let (_, exec) = mock_exec();

    // A bare definition: install/remove/list only, so no repo manager, not enumerable,
    // adoption skips it.
    let mut plain = BackendRegistry::new();
    register_custom_backends(&mut plain, &exec, vec![firewall_def()]);
    let caps = plain.get("firewall").unwrap();
    assert!(!caps.is_repo_manager(), "a bare def is not a repo manager");
    assert!(
        caps.as_enumerable().is_none(),
        "a bare def cannot list a catalogue"
    );

    // The same backend, now told how to manage repos and list its catalogue.
    let mut full = BackendRegistry::new();
    let def = CustomBackendDef {
        repo_add_args: Some(vec!["repo".into(), "add".into()]),
        repo_remove_args: Some(vec!["repo".into(), "rm".into()]),
        repo_list_args: Some(vec!["repo".into(), "list".into()]),
        repo_binary: None,
        repo_list_binary: None,
        enumerate_args: Some(vec!["list".into(), "--all".into()]),
        depends_args: Some(vec!["deps".into()]),
        manual: Some(ManualListingDef::AllInstalled),
        ..firewall_def()
    };
    register_custom_backends(&mut full, &exec, vec![def]);
    let caps = full.get("firewall").unwrap();
    assert!(
        caps.is_repo_manager(),
        "a def with repo args IS a repo manager"
    );
    assert!(
        caps.as_enumerable().is_some(),
        "a def with enumerate args CAN list its catalogue"
    );
}

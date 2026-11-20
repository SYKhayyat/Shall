use super::*;

/// A registry whose only backend is a `service` one answering a fixed list, so the **narrowing**
/// is the subject rather than the init. The real init table is exercised by
/// `backends::service::*`; what this pins is `C38`'s shape — the services the OS authored are
/// reported and not offered, and one flag takes them.
fn registry_of_services(names: &'static str) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_command_exists("systemctl", true);
    mock.set_response(
        "systemctl list-units",
        Ok(DryRunOutput {
            stdout: names.as_bytes().to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    let config = ManagerConfig {
        name: "service".into(),
        binary: Some("systemctl".into()),
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        list_args: vec![],
        manual: ManualListing::Command {
            binary: Some("systemctl".into()),
            args: vec!["list-units".into()],
            format: ManualFormat::BareNames,
        },
        essential_args: None,
        search_args: vec![],
        search_binary: None,
        enumerate_args: None,
        enumerate_binary: None,
        list_binary: None,
        upgrade_args: vec![],
        update_args: None,
        purge_args: None,
        orphan_dry_run: None,
        foreign_args: None,
        repo_add_args: None,
        repo_remove_args: None,
        repo_list_args: None,
        repo_binary: None,
        repo_list_binary: None,
        repo_remove_binary: None,
        repo_list_shape: shall::backends::generic::RepoListing::Columns,
        depends: None,
        clean_cache: None,
        version_pin: None,
        needs_root: false,
        is_exclusive: false,
        install_source_option: None,
        extra_probes: None,
        upgrade_reinstall_args: None,
        property_probes: Vec::new(),
        machine_list: None,
        outdated: None,
        search_source: SearchSource::Command,
        qualified_names: false,
    };
    let core = Arc::new(GenericBackendCore {
        name: "service".into(),
        executor: exec,
        config,
        parser: Arc::new(shall::parsers::LambdaParser {
            installed_fn: |out: &str| shall::parsers::parse_bare_names(out, "service"),
            search_fn: |_q: &str| Vec::new(),
        }),
    });
    let mut reg = BackendRegistry::new();
    reg.register(Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_queryable(Arc::new(GenericQueryable { core }))
            .build(),
    ));
    Arc::new(reg)
}

#[tokio::test]
async fn the_services_the_os_runs_for_itself_are_reported_and_not_offered() {
    // A bare `adopt` takes services (`C38`, amending `Q39`) — but not the ones the OS authored.
    // Authorship, not importance: a name on the `plumbing` list was put there by the installer
    // or by the tooling that boots the machine, so its presence is not evidence anybody chose
    // it. The alternative `Q39` ruled for was refusing the whole backend, which costs a
    // hand-written line for every service a person *did* choose.
    let m = adopter(registry_of_services(
        "dbus\nsystemd-journald.service\nnginx\ndocker\n",
    ));
    let found = m.discover_scoped(&AdoptScope::default()).await.unwrap();

    let adopted: Vec<&str> = found.adopt.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(adopted, vec!["docker", "nginx"], "what somebody chose");

    let left_out: Vec<&str> = found
        .skipped
        .iter()
        .map(|s| s.package.name.as_str())
        .collect();
    assert_eq!(
        left_out,
        vec!["dbus", "systemd-journald.service"],
        "the OS's own services, reported so the offer is complete minus a named set"
    );
    for s in &found.skipped {
        assert!(
            s.reason.contains("--all-services"),
            "the reason names the flag that takes it: {}",
            s.reason
        );
    }

    // `.service` is stripped before the comparison, so the two spellings of one service are one
    // entry in the list rather than two.
    let with_all = AdoptScope {
        all_services: true,
        ..AdoptScope::default()
    };
    let found = m.discover_scoped(&with_all).await.unwrap();
    let adopted: Vec<&str> = found.adopt.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        adopted,
        vec!["dbus", "docker", "nginx", "systemd-journald.service"],
        "`--all-services` takes them all, still spelled as the init spelled them"
    );
    assert!(found.skipped.is_empty(), "{:?}", found.skipped);
}

// C40: `adopt` offers the repositories this machine's managers list — and offers only the ones a
// `repo:` line can describe, because that line carries one string which the applying half hands
// to the manager as both its name and its source.

use super::*;
use shall::core::manager::RepoManager;

/// A manager that lists `name source` columns, as `brew source list` and `scoop bucket list` do.
fn manager_listing(name: &str, listing: &str) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_command_exists(name, true);
    mock.set_response(
        &format!("{} source list", name),
        Ok(DryRunOutput {
            stdout: listing.as_bytes().to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let config = ManagerConfig {
        name: name.into(),
        binary: None,
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        list_args: vec![],
        manual: ManualListing::AllInstalled,
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
        repo_add_args: Some(vec![
            "source".into(),
            "add".into(),
            "--name".into(),
            "{name}".into(),
            "--arg".into(),
            "{url}".into(),
        ]),
        repo_remove_args: Some(vec![
            "source".into(),
            "remove".into(),
            "--name".into(),
            "{name}".into(),
        ]),
        repo_list_args: Some(vec!["source".into(), "list".into()]),
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
        name: name.into(),
        executor: exec,
        config,
        parser: Arc::new(shall::parsers::LambdaParser {
            installed_fn: |out: &str| shall::parsers::parse_bare_names(out, "plain"),
            search_fn: |_q: &str| Vec::new(),
        }),
    });
    let mut reg = BackendRegistry::new();
    reg.register(Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_queryable(Arc::new(GenericQueryable { core: core.clone() }))
            .with_repo_manager(Arc::new(shall::backends::generic::GenericRepoManager {
                core,
            }))
            .build(),
    ));
    Arc::new(reg)
}

fn adopter(reg: Arc<BackendRegistry>) -> Adopter {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    let config = Config::default();
    Adopter::new(
        over(reg),
        Arc::new(Mutex::new(StateRegistry::default())),
        &config,
        exec,
    )
}

#[tokio::test]
async fn a_repository_whose_name_is_its_source_is_offered_and_one_that_differs_is_named() {
    // **A `repo:` line carries ONE string, and the applying half hands it to the manager as both
    // the name and the source** (`repositories.rs`), so a repository whose two differ has no
    // line: written now it would re-add it at the wrong address. That is most of what `brew
    // source list` prints — `homebrew/core` pointing at a GitHub URL — and each one is reported
    // with both strings rather than offered with one of them. A source that is its own name
    // (a manager keyed by URL) IS a line, and that is the row offered here.
    let m = adopter(manager_listing(
        "brew",
        "homebrew/core        https://github.com/Homebrew/homebrew-core\n\
         https://mirror.example/tap    https://mirror.example/tap\n",
    ));
    let found = m.discover_scoped(&AdoptScope::default()).await.unwrap();

    let offered: Vec<String> = found
        .repositories
        .iter()
        .map(|r| format!("repo:{}:{}", r.backend, r.spec))
        .collect();
    assert_eq!(
        offered,
        vec!["repo:brew:https://mirror.example/tap"],
        "the repository whose name and source are one string is a line"
    );

    let named = found
        .skipped
        .iter()
        .find(|s| s.package.name == "homebrew/core")
        .expect("the one that differs is reported, not silently dropped");
    assert!(
        named.reason.contains("one string for both"),
        "the reason says why no line can hold it: {}",
        named.reason
    );

    let manifest = m.render_manifest(&found, &HashMap::new());
    assert!(
        manifest
            .lines()
            .any(|l| l == "repo:brew:https://mirror.example/tap"),
        "the manifest carries it as a live line: {}",
        manifest
    );
    assert!(
        !manifest
            .lines()
            .any(|l| l.starts_with("repo:brew:homebrew/core")),
        "and never as a line it cannot honour: {}",
        manifest
    );
}

#[tokio::test]
async fn a_manager_that_cannot_list_its_repositories_is_named_rather_than_read_as_empty() {
    // `apt` has no repository listing at all (`repo_list_args: None`), so its answer is an
    // error. An error is not "no repositories": a manifest that says nothing about apt's
    // sources reads as a machine with none, and a PPA nobody declared is exactly the missing
    // line V.253 argues about.
    let m = adopter(Arc::new(BackendRegistry::default()));
    let found = m.discover_scoped(&AdoptScope::default()).await.unwrap();
    assert!(found.repositories.is_empty());
}

#[tokio::test]
async fn user_added_refuses_by_name_when_no_base_table_exists() {
    // `--user-added` compares against the base sources a distribution ships, and no such table
    // is kept yet. A filter that cannot be computed must not answer, so the flag says so and
    // offers nothing — rather than offering everything under a label that promises a filter.
    // No manager is asked at all — the refusal comes before the crawl, because a run that
    // offered repositories under a label promising a filter would be worse than one that
    // offered none. The property under test is the refusal, so this drives a machine with no
    // manager that could answer.
    let m = adopter(Arc::new(BackendRegistry::default()));
    let scope = AdoptScope {
        user_added: true,
        user_only: true,
        ..AdoptScope::default()
    };
    let found = m.discover_scoped(&scope).await.unwrap();

    assert!(
        found.repositories.is_empty(),
        "--user-added offers nothing it cannot filter: {:?}",
        found.repositories
    );
    let why = found
        .skipped_backends
        .iter()
        .find(|(name, _)| name == "repo")
        .expect("the refusal is reported");
    assert!(
        why.1.contains("--user-added"),
        "the reason names the flag: {}",
        why.1
    );
}

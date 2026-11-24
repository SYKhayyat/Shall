//! **The two install hooks fire, per package, and a failing `before_install` drops only its own.**
//!
//! Nothing drove a hook through a transaction. `fire_after_install` gutted to `()` — the whole
//! function replaced by nothing — survived the entire suite, because every hook test calls
//! `LuaHooks::run_hook` directly and none of them asks whether a *run* ever calls it. A hook that
//! is declared, approved and then never fired is the exact shape `P3` names: the feature is on
//! the page and absent from the machine.
//!
//! The second claim is the one with a decision in it: `before_install` is allowed to *refuse* a
//! member, and refusal must be local. A hook that fails for one package takes that package out of
//! the batch and leaves the rest of the command alone — anything else turns one package's
//! prerequisite into everybody's.
//!
//! **One marker file per package, never one file for the whole run.** Both hooks of a batch are
//! invoked concurrently (`buffer_unordered`), and the first version of this test appended to a
//! single file: on Windows `Add-Content` lost one of the two writes to the other's handle, so the
//! test read one line and concluded the hook had not been called. A per-package file has no two
//! writers, so "was this package asked?" is answered by the filesystem rather than by a race.

use shall::app::hooks::LuaHooks;
use shall::core::executor::DryRunOutput;
use shall::core::{GraphAction, PackageSpec, Transaction, TransactionConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::mock_providers::TestKernel;

/// A script that records the package it was told about as `dir/<name>.txt`, and optionally exits
/// non-zero for one named package — **after** writing, so the file answers "was this package
/// asked?" separately from "did the hook say yes?".
fn recording(dir: &Path, refuse: Option<&str>) -> String {
    if cfg!(windows) {
        let refuse = refuse
            .map(|n| format!("if ($env:SHALL_PKG_NAME -eq '{n}') {{ exit 3 }};"))
            .unwrap_or_default();
        format!(
            "#!/usr/bin/env powershell\nSet-Content -Path (Join-Path '{}' ($env:SHALL_PKG_NAME + '.txt')) -Value $env:SHALL_PKG_NAME\n{refuse}\n",
            dir.display()
        )
    } else {
        let refuse = refuse
            .map(|n| format!("if [ \"$SHALL_PKG_NAME\" = '{n}' ]; then exit 3; fi\n"))
            .unwrap_or_default();
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$SHALL_PKG_NAME\" > '{}/'\"$SHALL_PKG_NAME\"'.txt'\n{refuse}",
            dir.display()
        )
    }
}

/// Which package a hook was told about, or `None` if it was never asked.
fn asked(dir: &Path, package: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(format!("{package}.txt")))
        .ok()
        .map(|s| s.trim().to_string())
}

fn install(name: &str) -> GraphAction {
    GraphAction::Install(PackageSpec {
        name: name.into(),
        backend: "brew".into(),
        options: Default::default(),
        requires: vec![],
        present: true,
    })
}

fn run(
    kernel: &TestKernel,
    graph: petgraph::stable_graph::StableDiGraph<GraphAction, ()>,
    hooks: LuaHooks,
) -> Transaction {
    Transaction::with_config(
        graph,
        kernel.app.registry.clone(),
        kernel.app.journal.clone(),
        kernel.app.diagnostics.clone(),
        kernel.app.config.clone(),
        TransactionConfig::default(),
    )
    .with_hooks(Arc::new(hooks))
}

fn hook_dir(root: &Path, hook: &str) -> PathBuf {
    let dir = root.join(hook);
    std::fs::create_dir_all(&dir).expect("the marker directory");
    dir
}

/// Both hooks run, each told which package it is about — the four facts reach a hook fired from a
/// run the same way they reach one called by hand.
#[tokio::test]
async fn both_install_hooks_fire_and_are_told_which_package() {
    let kernel = TestKernel::new().await;
    let temp = tempfile::tempdir().expect("a temp dir");
    let before = hook_dir(temp.path(), "before");
    let after = hook_dir(temp.path(), "after");

    let mut config = shall::config::Config::sandboxed(kernel.tmp.path());
    config.hooks.insert(
        "before_install".to_string(),
        HashMap::from([("hooked".to_string(), recording(&before, None))]),
    );
    config.hooks.insert(
        "after_install".to_string(),
        HashMap::from([("hooked".to_string(), recording(&after, None))]),
    );
    let hooks = LuaHooks::new(&config).expect("hooks");

    kernel
        .mock_executor
        .set_response("brew install -- hooked", Ok(DryRunOutput::default().into()));

    let mut graph = petgraph::stable_graph::StableDiGraph::new();
    graph.add_node(install("hooked"));

    run(&kernel, graph, hooks)
        .execute_with_telemetry()
        .await
        .expect("the install failed");

    for (what, dir) in [("before_install", &before), ("after_install", &after)] {
        assert_eq!(
            asked(dir, "hooked").as_deref(),
            Some("hooked"),
            "the {what} hook never ran for a package the run installed, or was not told which one"
        );
    }
}

/// A `before_install` that refuses one package refuses it alone: the other member installs, and
/// the refused one never reaches an install command.
#[tokio::test]
async fn a_before_install_that_refuses_one_package_leaves_the_rest_of_the_batch() {
    let kernel = TestKernel::new().await;
    let temp = tempfile::tempdir().expect("a temp dir");
    let before = hook_dir(temp.path(), "before");
    let after = hook_dir(temp.path(), "after");

    let mut config = shall::config::Config::sandboxed(kernel.tmp.path());
    config.hooks.insert(
        "before_install".to_string(),
        HashMap::from([
            ("kg-good".to_string(), recording(&before, Some("kg-bad"))),
            ("kg-bad".to_string(), recording(&before, Some("kg-bad"))),
        ]),
    );
    config.hooks.insert(
        "after_install".to_string(),
        HashMap::from([
            ("kg-good".to_string(), recording(&after, None)),
            ("kg-bad".to_string(), recording(&after, None)),
        ]),
    );
    let hooks = LuaHooks::new(&config).expect("hooks");

    // One command, for the member the hook kept. `kg-bad` is refused before this is built, so a
    // command naming it would be the defect this test exists for.
    kernel.mock_executor.set_response(
        "brew install -- kg-good",
        Ok(DryRunOutput::default().into()),
    );

    let mut graph = petgraph::stable_graph::StableDiGraph::new();
    graph.add_node(install("kg-good"));
    graph.add_node(install("kg-bad"));

    let result = run(&kernel, graph, hooks).execute_with_telemetry().await;
    let said = match result {
        Err(e) => e.to_string(),
        Ok(telemetry) => panic!("a refused member reported success: {telemetry:?}"),
    };
    // The run fails, and says which hook. It does **not** say which package — see `#223`; the
    // assertion here is only the part the code promises, so that the package name can be added
    // without this test having to bless the omission.
    assert!(
        said.contains("before_install"),
        "the refusal does not name the hook that refused: {said}"
    );

    let calls = kernel.mock_executor.get_calls().await;
    let installed_bad = |c: &String| c.starts_with("brew install") && c.contains("kg-bad");
    assert!(
        !calls.iter().any(installed_bad),
        "the refused member reached an install command anyway: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "brew install -- kg-good"),
        "the hook's refusal took the other member down with it: {calls:?}"
    );

    assert_eq!(
        asked(&before, "kg-good").as_deref(),
        Some("kg-good"),
        "the member that installed was never asked about before it was"
    );
    assert_eq!(
        asked(&before, "kg-bad").as_deref(),
        Some("kg-bad"),
        "the refused member was never asked either, so nothing refused it"
    );
    assert_eq!(
        asked(&after, "kg-good").as_deref(),
        Some("kg-good"),
        "after_install did not fire for the member that installed"
    );
    assert!(
        asked(&after, "kg-bad").is_none(),
        "after_install fires for the members that were installed, and only those"
    );
}

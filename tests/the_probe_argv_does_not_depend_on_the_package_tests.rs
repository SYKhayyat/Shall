//! Is a property probe's *subprocess* asked once per run, or once per package?
//!
//! `GenericQueryable::info` enriches the package it found with whatever the backend's
//! `property_probes` rows can add — npm's `npm prefix -g`, pnpm's `pnpm bin -g`, pipx's
//! `pipx environment --value PIPX_HOME`. Each row runs a real subprocess per call, and the outer
//! fan-outs ask `info` about many packages at once (`planner.rs`, `resolver.rs`,
//! `transaction.rs`), so one `shall info` over twenty packages ran twenty `npm prefix -g`
//! (`PLAN.md` #80).
//!
//! **The argv cannot depend on the package, and that is not a hope — it is where `{name}` goes.**
//! `PropertyProbe::resolve` substitutes `{base}` and `{name}` into the probe's **template**, and
//! passes `args` to the subprocess untouched; `walk_args` visits `args` and `template` for
//! *settings* placeholders only. So the answer depends on `cmd` + `args` and on nothing else, and
//! a memo keyed by the argv is keyed by everything the answer depends on. Which is also why the
//! npm row is the right fixture: its template is `{base}/lib/node_modules/{name}` — package-named
//! in the answer, package-blind in the question.
//!
//! **And the memo is dropped by a mutation**, because a memo that outlives the thing it memoised
//! is the shape this repository has now found in the guard, in the run-scoped listings and in
//! `forget_all`: an invalidation that covers one layer and not the other covers neither. That is
//! asserted through the product — a real `npm install` — rather than by calling the invalidator.

use shall::backends::create_default_registry;
use shall::config::grammar::Options;
use shall::core::executor::DryRunOutput;
use shall::core::executor::MockExecutor;
use shall::core::{CommandExecutor, Package, PackageSpec};
use std::collections::HashMap;
use std::sync::Arc;

const LISTING: &str = r#"{
  "name": "lib",
  "dependencies": {
    "alpha": { "version": "1.0.0", "overridden": false },
    "beta":  { "version": "2.0.0", "overridden": false },
    "gamma": { "version": "3.0.0", "overridden": false }
  }
}"#;

/// An npm whose `prefix -g` answers `/usr/lib`, wired through the real builtin row.
async fn npm_with_a_prefix() -> (Arc<MockExecutor>, Arc<shall::backends::BackendRegistry>) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response("npm list -g --depth=0 --json", Ok(answer(LISTING)));
    mock.set_response("npm prefix -g", Ok(answer("/usr/lib")));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let cfg = shall::config::Config::default();
    let hooks = Arc::new(shall::app::LuaHooks::new(&cfg).expect("hooks init"));
    let reg = Arc::new(create_default_registry(exec, &cfg, hooks).await);
    (mock, reg)
}

/// A successful exit with `stdout`, the shape every other test in the suite builds.
fn answer(stdout: &str) -> shall::core::executor::StdOutput {
    DryRunOutput {
        stdout: stdout.as_bytes().to_vec(),
        stderr: vec![],
    }
    .into()
}

fn times(haystack: &[String], needle: &str) -> usize {
    haystack.iter().filter(|c| c.as_str() == needle).count()
}

fn property<'a>(pkg: &'a Package, key: &str) -> Option<&'a String> {
    pkg.properties
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v))
}

/// **The finding, on the shipped row.** Three `info` calls, one `npm prefix -g`.
#[tokio::test]
async fn one_prefix_query_serves_every_package_asked_about() {
    let (mock, reg) = npm_with_a_prefix().await;
    let npm = reg.get("npm").expect("npm registers");
    let q = npm.as_queryable().expect("npm answers questions");

    for name in ["alpha", "beta", "gamma"] {
        q.info(name)
            .await
            .expect("info answers")
            .expect("alpha is installed");
    }

    let calls = mock.get_calls().await;
    assert_eq!(
        times(&calls, "npm prefix -g"),
        1,
        "`npm prefix -g` is asked {n} times for three packages, and its answer cannot depend on \
         which package was asked about: the probe substitutes `{{name}}` into its template, never \
         into its argv. One question, one answer, one run. Calls:\n  {calls:#?}",
        n = times(&calls, "npm prefix -g"),
    );
}

/// **The family, and the half a memo could plausibly break.** The prefix is shared; the
/// *installed path* is per package. A memo that collapsed the whole answer would satisfy the test
/// above and lose the property, so both are asserted on the same three calls.
#[tokio::test]
async fn a_shared_probe_still_answers_per_package() {
    let (_mock, reg) = npm_with_a_prefix().await;
    let npm = reg.get("npm").expect("npm registers");
    let q = npm.as_queryable().expect("npm answers questions");

    let mut paths = Vec::new();
    for name in ["alpha", "beta", "gamma"] {
        let pkg = q
            .info(name)
            .await
            .expect("info answers")
            .expect("the package is installed");
        paths.push((
            name.to_string(),
            property(&pkg, "install_path")
                .cloned()
                .expect("npm's row declares install_path"),
        ));
    }
    assert_eq!(
        paths,
        vec![
            (
                "alpha".to_string(),
                "/usr/lib/lib/node_modules/alpha".to_string()
            ),
            (
                "beta".to_string(),
                "/usr/lib/lib/node_modules/beta".to_string()
            ),
            (
                "gamma".to_string(),
                "/usr/lib/lib/node_modules/gamma".to_string()
            ),
        ],
        "the per-package half of the answer changed; a shared question must not become a shared \
         answer"
    );
}

/// **A mutation drops it, asked through the product.** This is the assertion that makes the memo
/// safe rather than merely fast: `forget_run_scoped_answers` runs after every mutating command, so
/// the second question after an install is asked again — and an implementation that forgot to wire
/// the invalidation would answer `shall info` from a prefix taken before the install.
#[tokio::test]
async fn a_mutation_makes_the_next_question_a_question_again() {
    let (mock, reg) = npm_with_a_prefix().await;
    let npm = reg.get("npm").expect("npm registers");
    let q = npm.as_queryable().expect("npm answers questions");

    q.info("alpha").await.expect("info answers");
    q.info("beta").await.expect("info answers");
    let before = times(&mock.get_calls().await, "npm prefix -g");
    assert_eq!(
        before, 1,
        "the memo never engaged, so this test proves nothing"
    );

    let mut options = Options::default();
    options.set("version".to_string(), "1.0.0".to_string());
    npm.as_installable()
        .expect("npm installs")
        .install(
            &[PackageSpec {
                name: "alpha".into(),
                backend: "npm".into(),
                options,
                ..Default::default()
            }],
            false,
        )
        .await
        .expect("npm installs");

    q.info("beta")
        .await
        .expect("info answers after the install");
    let after = times(&mock.get_calls().await, "npm prefix -g");
    assert_eq!(
        after, 2,
        "the prefix was asked {after} times after a mutation (it was asked {before} before one). \
         A run-scoped memo that outlives the mutation it was taken across is the invalidation \
         that covers one layer and not the other — the shape this repo has found in the guard, in \
         the run-scoped listings and in `forget_all`."
    );
    // A second `HashMap` import kept for the config type the row resolver wants.
    let _ = HashMap::<String, String>::new();
}

/// **The window a `once`-then-forget test cannot reach, and what it does and does not prove.** A
/// question already in flight when the mutation lands is holding a slot the forget cannot reach —
/// it cloned the `Arc` before the clear — so it stores its answer *after* the invalidation. This
/// asks the question with the install landing **while `npm prefix -g` is running**, and requires
/// that the question after the install is still asked.
///
/// **It does not isolate the round stamp, and that is worth saying rather than implying.** Two
/// mutations were tried against it: dropping the stamp check leaves it green, because
/// `forget_all` bumps the generation and clears the map adjacently, so the late write lands in a
/// slot no caller can reach any more and the next caller gets a fresh one. The stamp is kept
/// because it is the defence `once` has and the two memos should not differ in their one safety
/// property — but the claim here is the narrow one this test establishes: **a mutation landing
/// mid-question does not make the question after it free.**
///
/// The ordering is waited for rather than slept through — `MockExecutor` logs a call before it
/// delays it, so "the probe has started" is observable — because a fixed sleep would make this
/// test a coin toss on a loaded machine, and a flaky test is worse than an honest gap.
#[tokio::test]
async fn a_mutation_landing_mid_question_does_not_leave_that_answer_current() {
    let (mock, reg) = npm_with_a_prefix().await;
    let npm = reg.get("npm").expect("npm registers");
    mock.set_delay("npm prefix -g", std::time::Duration::from_millis(400));

    let asked = npm.clone();
    let inflight = tokio::spawn(async move {
        asked
            .as_queryable()
            .expect("npm answers questions")
            .info("alpha")
            .await
    });

    // Wait for the probe to be under way rather than guessing how long that takes.
    for _ in 0..500 {
        if times(&mock.get_calls().await, "npm prefix -g") >= 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(
        times(&mock.get_calls().await, "npm prefix -g"),
        1,
        "the probe never started, so the mutation below is not landing mid-question"
    );

    // The mutation, while the probe is still answering.
    let mut options = Options::default();
    options.set("version".to_string(), "1.0.0".to_string());
    npm.as_installable()
        .expect("npm installs")
        .install(
            &[PackageSpec {
                name: "alpha".into(),
                backend: "npm".into(),
                options,
                ..Default::default()
            }],
            false,
        )
        .await
        .expect("npm installs");
    inflight
        .await
        .expect("the in-flight question finished")
        .expect("the in-flight question answered");

    let npm = reg.get("npm").expect("npm registers");
    npm.as_queryable()
        .expect("npm answers questions")
        .info("beta")
        .await
        .expect("info answers after the install");

    assert_eq!(
        times(&mock.get_calls().await, "npm prefix -g"),
        2,
        "the question that was already running when the mutation landed was served to the caller \
         after it, and the question after the install was never asked"
    );
}

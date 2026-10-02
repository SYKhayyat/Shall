//! Does a mutation forget the managers it could have changed, and only those?
//!
//! **`InstalledListings::forget_manager` existing is not the same as the product reaching it.**
//! Three layers have to line up: the setting has to survive config into the shared memo, the one
//! call site that is handed a *manager* has to pass it, and the memo has to drop the whole lock
//! family rather than the one backend whose name happened to be typed. `core::installed`'s own
//! tests cover the third; this file covers the first two, end to end through `run_exclusive`
//! (`PLAN.md` #79).
//!
//! **Both directions are asserted from the same two lines of setup**, because a setting that only
//! works when set is not a default. Y6's ruling is `all`, so that is what an unset scope must do.

use shall::core::executor::{DryRunOutput, MockExecutor};
use shall::core::{CommandExecutor, Package};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<dashmap::DashMap<PathBuf, String>> = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let executor = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    (executor, mock)
}

/// Prime one backend's listing, install through `manager`, then ask `backend` again — and report
/// whether it had to be asked again.
async fn survives_a_mutation_through(
    scope: Option<shall::config::config::InvalidationScope>,
    manager: &str,
    program: &str,
    args: &[&str],
    backend: &'static str,
) -> bool {
    let (executor, mock) = wired();
    if let Some(scope) = scope {
        executor.installed_listings().set_scope(scope);
    }
    let command = format!("{program} {}", args.join(" "));
    mock.set_response(&command, Ok(DryRunOutput::new().into()));

    let fetches = Arc::new(AtomicUsize::new(0));
    let listing = |name: &'static str| {
        let fetches = fetches.clone();
        async move {
            fetches.fetch_add(1, Ordering::SeqCst);
            Ok(vec![Package::new(name, "1")])
        }
    };
    executor
        .installed_listings()
        .once(backend, listing("jq"))
        .await
        .expect("the first listing");

    executor
        .run_exclusive(manager, program, args, false)
        .await
        .unwrap_or_else(|e| panic!("`{command}` ran: {e}"));

    executor
        .installed_listings()
        .once(backend, listing("jq"))
        .await
        .expect("the second listing");
    fetches.load(Ordering::SeqCst) == 2
}

/// **The ruling, unchanged.** An `apt` install forgets everything, so `brew` is asked again.
#[tokio::test]
async fn without_the_setting_a_mutation_re_lists_every_manager() {
    assert!(
        survives_a_mutation_through(None, "apt", "apt", &["install", "-y", "jq"], "brew").await,
        "with no setting — the ruling's scope — an `apt` install left `brew`'s listing standing"
    );
}

/// **And what the setting is for.** Same two lines, and `brew` keeps its listing: an `apt`
/// mutation cannot have changed what `brew` has installed.
#[tokio::test]
async fn the_narrow_scope_keeps_a_manager_the_mutation_cannot_have_changed() {
    assert!(
        !survives_a_mutation_through(
            Some(shall::config::config::InvalidationScope::MutatedManager),
            "apt",
            "apt",
            &["install", "-y", "jq"],
            "brew",
        )
        .await,
        "under `mutated_manager` an `apt` install still re-listed `brew`, so the setting bought \\
         nothing"
    );
}

/// **And the family still goes together**, which is the half a per-backend reading gets wrong:
/// `yay` is a different program from `pacman` over one database, so keeping `yay`'s listing after
/// an install through `pacman` is stale by definition.
#[tokio::test]
async fn the_narrow_scope_still_drops_a_sibling_backend_on_the_same_lock() {
    assert!(
        survives_a_mutation_through(
            Some(shall::config::config::InvalidationScope::MutatedManager),
            "pacman",
            "pacman",
            &["-S", "jq"],
            "yay",
        )
        .await,
        "`yay` shares `pacman`'s manager lock, so an install through `pacman` must drop its \\
         listing too; a narrow forget that forgot one backend rather than one lock would serve a \\
         listing taken across the install"
    );
}

/// **The scope's default is the ruling**, said here as well as in `core::installed` because this is
/// the layer a user's config actually reaches, and a default that quietly flipped would be the
/// whole harm Y6's ruling exists to prevent.
#[test]
fn the_default_scope_is_the_ruling() {
    assert_eq!(
        shall::config::Config::default().listing_invalidation,
        shall::config::config::InvalidationScope::All,
        "the default scope is not `all`, so Y6's ruling is not what an unconfigured Shall does"
    );
}

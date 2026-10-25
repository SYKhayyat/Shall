//! What each manager has installed, asked once per run.
//!
//! Answering "is `jq` installed?" by listing every package the manager has is what nearly every
//! backend in this tree does — `info` is `list_installed()` plus a `find` in eighteen of them —
//! and the callers ask it once per *declared* package. Measured on Windows, `shall check drift`
//! cost ~247 ms more for every additional declaration, on a command whose whole job is to
//! compare two lists it could have fetched once. On Ubuntu the same shape produced exactly
//! `declared + 1` `dpkg-query` invocations.
//!
//! The listing is the same answer every time within one command, so it is fetched once and
//! reused. A mutation is the one thing that can change it, and every mutation goes through
//! `CommandExecutor::run`, which forgets these.
//!
//! **Per executor, not per process.** Every backend of one `App` shares that `App`'s executor,
//! so the memo is scoped to the run — which is also what keeps one test's mock listing out of
//! the next test's, in a suite where a hundred `App`s live in one process.
//!
//! **And optionally per machine, across runs** (`installed_cache_secs`, off by default). Asking
//! once per run is the whole win while a run lasts; the next `shall list` still pays ~3.2 s to
//! ask 24 managers the same question about a machine nothing has touched. The disk layer sits
//! behind this same seam, so a backend cannot tell the difference and no caller had to change.

use crate::core::{Package, Result};
use dashmap::DashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// One manager's listing, and whether it has been fetched yet.
/// The listing itself is behind an `Arc` as well as the slot: **`once` is asked per
/// package, not per backend**, and every hit used to clone the whole `Vec<Package>` out.
/// Measured on a 256-line winget config against a 280-package listing, that is ~71,680
/// `Package` clones to answer 256 questions.
type Listing = Arc<Vec<Package>>;
/// A slot remembers which generation of the memo its answer belongs to. See
/// [`InstalledListings::forget_all`] for why the generation is what makes the clear complete.
type Slot = Arc<tokio::sync::Mutex<Option<(u64, Listing)>>>;

/// A listing as it sits on disk, with the moment it was taken.
#[derive(serde::Serialize, serde::Deserialize)]
/// **`pub` for the integration tests' sake.** The cache mirror links the library without
/// `cfg(test)` and must deserialize the exact on-disk record rather than a second shape.
#[doc(hidden)]
pub struct CachedListing {
    /// Seconds since the epoch. Stored rather than read off the file's mtime, because a
    /// backup, a sync tool or a `cp -p` rewrites mtimes and none of them refreshed the answer.
    pub taken_at: u64,
    pub packages: Vec<Package>,
}

/// One manager's essential set, shared the same way a listing is.
type Essentials = Arc<Vec<String>>;
type EssentialSlot = Arc<tokio::sync::Mutex<Option<(u64, Essentials)>>>;

#[derive(Default)]
pub struct InstalledListings {
    by_backend: DashMap<String, Slot>,
    /// What each manager reports as OS-essential, asked once per run.
    ///
    /// **The sibling of `by_backend` that never got the memo.** `essential()` is a live
    /// subprocess per backend and `guard::essential_names` is on every removal path — its own
    /// comment says so. A single `sync` with removals goes through the preview path and the
    /// enforce path, so the whole set ran at least twice; a rollback made it three times. The
    /// argument `list_installed` makes applies unchanged: the answer cannot change while
    /// nothing is being installed, and the one thing that can change it is a mutating command,
    /// which already calls `forget_all`. Kept here rather than in a second map with a second
    /// policy, so it inherits that invalidation — including the generation counter — instead
    /// of inventing its own and its own version of R2's staleness window.
    ///
    /// No disk layer: this is run-scoped only. `installed_cache_secs` is a bargain about a
    /// *report* being stale, and the essential set exists to refuse removals.
    essentials: DashMap<String, EssentialSlot>,
    /// Which round of answers is current. Bumped by every invalidation.
    ///
    /// **Clearing the map is not enough, because the map is not what a waiting task holds.**
    /// `once` clones the `Arc<Slot>` out of the `DashMap` *before* it waits on the slot's
    /// mutex, and it waits there for the length of a real listing subprocess — over a second
    /// on Windows, and the whole point of holding the lock across the fetch is that two askers
    /// produce one call. A mutating command completing during that wait calls `forget_all`,
    /// which drops the map entry and cannot touch the `Arc` already handed out; the waiter
    /// then wins the lock and returns a listing that predates both the install and the
    /// invalidation meant to cover it. Exposure grows with `max_parallel`, so it is worst on
    /// the configurations that matter most.
    ///
    /// The generation goes *in the answer*, which is how `VARS_MEMO`/`RESOLUTION` in
    /// `app::sync::resolver` solves the identical problem: an invalidated entry cannot be
    /// reached by an already-cloned handle, because the handle carries the round it came from.
    generation: std::sync::atomic::AtomicU64,
    /// How long a listing on disk stays usable. `None` — the default — is no disk layer.
    ttl: Option<Duration>,
    /// How wide a mutation's invalidation reaches. A `OnceLock` because it is config, set once
    /// before the run and read from every clone of every backend's executor: unset means the
    /// ruling's scope, so a test that builds this type directly is never accidentally narrow.
    scope: std::sync::OnceLock<crate::config::config::InvalidationScope>,
    /// Questions that are not listings and are not essentials, and are asked once per run anyway.
    ///
    /// **Here rather than beside it, for the reason `essentials` gives above: a second map with a
    /// second policy invents a second version of R2's staleness window.** A backend's
    /// `property_probes` row runs a subprocess whose answer cannot depend on which package asked
    /// (`PLAN.md` #80), so `queries` is a listing-shaped thing with a listing's lifetime and
    /// inherits this type's `generation` and this type's `forget_all` — including the bump that
    /// makes an answer taken across a mutation stale on arrival, which is the half a second round
    /// counter would have had to be argued into existence for.
    ///
    /// No disk layer, for the same reason `essentials` has none: this is run-scoped only, and
    /// `installed_cache_secs` is a bargain about a *report* being stale.
    queries: DashMap<String, QuerySlot>,
}

/// A question's answer, and the round it was taken in. See [`InstalledListings::generation`].
type QuerySlot = Arc<tokio::sync::Mutex<Option<(u64, String)>>>;

impl InstalledListings {
    pub fn new() -> Self {
        Self::default()
    }

    /// [`crate::config::Config::listing_invalidation`], set once from config before any backend
    /// holds a clone of the executor.
    pub fn set_scope(&self, scope: crate::config::config::InvalidationScope) {
        let _ = self.scope.set(scope);
    }

    /// What a mutation invalidates. `manager` is the lock key of the manager just run, and only
    /// `run_exclusive` has one — a command position holds the *program*, and for a user-defined
    /// backend those are different strings (XIII.12) — so every other caller passes `None` and
    /// forgets everything rather than guessing (`PLAN.md` #79).
    pub fn forget_after_mutation(&self, manager: Option<&str>) {
        match (self.scope.get().copied(), manager) {
            (Some(crate::config::config::InvalidationScope::MutatedManager), Some(manager)) => {
                self.forget_manager(manager)
            }
            _ => self.forget_all(),
        }
    }

    /// Whether a run of `subcommand` may be answered from disk at all.
    ///
    /// **A cached listing may inform a report; it may never source a decision that outlives the
    /// run.** The listing is up to `installed_cache_secs` old, and the whole bargain is that a
    /// stale answer costs you a stale *reading* — which the next run corrects. That bargain
    /// stops holding the moment the answer is written down:
    ///
    /// - a plan built from a listing taken before the user removed something by hand skips the
    ///   install and reports success, leaving a declared package absent and nothing saying so;
    /// - `adopt` writes a declaration for a package that is no longer there, and the next
    ///   `sync` installs it back;
    /// - `plan --out` freezes that same mistake into a file `apply` runs later.
    ///
    /// An allowlist rather than a list of the unsafe ones, because the next command added to
    /// Shall should have to say it is a reader — not discover it was assumed to be one.
    pub fn cache_may_answer(subcommand: &str) -> bool {
        matches!(
            subcommand,
            "list" | "search" | "check" | "outdated" | "info" | "why"
        )
    }

    /// Reuse listings across runs for `secs`. Zero keeps the memo run-scoped, as before.
    pub fn with_ttl(secs: u64) -> Self {
        Self {
            ttl: (secs > 0).then(|| Duration::from_secs(secs)),
            ..Self::default()
        }
    }

    /// **`pub` for the integration tests' sake.** The cache mirror links the library without
    /// `cfg(test)` and must inspect the path selected by production code.
    #[doc(hidden)]
    pub fn cache_dir() -> PathBuf {
        crate::utils::safe_data_dir()
            .join("cache")
            .join("installed")
    }

    /// One file per manager, so a mutation through one never invalidates the others and a
    /// half-written file can only cost the manager it belongs to.
    ///
    /// The name is sanitised because a backend name reaches this from the registry, and a
    /// `..` there would write outside the cache directory.
    /// **`pub` for the integration tests' sake.** The cache mirror links the library without
    /// `cfg(test)` and must inspect the path selected by production code.
    #[doc(hidden)]
    pub fn cache_file(backend: &str) -> Option<PathBuf> {
        let safe: String = backend
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        (!safe.is_empty()).then(|| Self::cache_dir().join(format!("{}.json", safe)))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// The listing on disk, if there is one and it is younger than the TTL.
    ///
    /// Every failure here — unreadable, unparseable, a clock that moved backwards — answers
    /// `None` and asks the manager. A cache that cannot be read is a cache miss; it is never a
    /// reason to fail a command, and never a reason to report a machine as empty.
    ///
    /// The read runs on the blocking pool: it happens behind the slot mutex inside an async
    /// fn, and a multi-megabyte listing read on a runtime worker parks everything scheduled
    /// behind it for the length of the disk hit.
    async fn read_from_disk(&self, backend: &str) -> Option<Vec<Package>> {
        let ttl = self.ttl?;
        let path = Self::cache_file(backend)?;
        let raw =
            crate::core::blocking::off_the_runtime(move || std::fs::read_to_string(path).ok())
                .await
                .ok()??;
        let cached: CachedListing = serde_json::from_str(&raw).ok()?;
        let age = Self::now_secs().checked_sub(cached.taken_at)?;
        (age <= ttl.as_secs()).then_some(cached.packages)
    }

    async fn to_disk(&self, backend: &str, packages: &[Package]) {
        if self.ttl.is_none() {
            return;
        }
        let Some(path) = Self::cache_file(backend) else {
            return;
        };
        let entry = CachedListing {
            taken_at: Self::now_secs(),
            packages: packages.to_vec(),
        };
        // Serialised on the caller's worker (cheap, in-memory); only the write moves.
        let Ok(json) = serde_json::to_string(&entry) else {
            return;
        };
        let dir = Self::cache_dir();
        // A failed cache write is ignored on purpose — same contract as the read side: the
        // cache is best-effort, and the manager is asked again next time either way.
        crate::core::blocking::off_the_runtime(move || {
            std::fs::create_dir_all(&dir).is_ok()
                // Written through a temp file and renamed: a listing half-flushed when the
                // process is killed would otherwise be read back as a shorter machine, and a
                // shorter machine is a list of things to remove.
                //
                // The temp name carries the pid, because the rename is only atomic per writer:
                // two `shall` runs sharing one temp path write into each other's file and rename
                // the interleaving, which is the torn listing this exists to prevent, arrived at
                // by the mechanism meant to prevent it. A prompt hook and a terminal are two runs.
                && {
                    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
                    if std::fs::write(&tmp, json).is_ok() && std::fs::rename(&tmp, &path).is_err() {
                        let _ = std::fs::remove_file(&tmp);
                    }
                    true
                }
        })
        .await
        .ok();
    }

    /// Drop every listing this machine has on disk.
    ///
    /// Public because a mutation is not the only thing that invalidates one: `shall clean-cache`
    /// exists for the case where something outside Shall changed the machine and the user knows
    /// it before the TTL does.
    pub fn forget_on_disk() -> std::io::Result<usize> {
        let dir = Self::cache_dir();
        if !dir.exists() {
            return Ok(0);
        }
        let mut dropped = 0;
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            // `.tmp` too: a run killed between the write and the rename leaves one, and the
            // command whose job is "empty the cache" is the only thing that ever looks here.
            let ours = path.extension().is_some_and(|e| e == "json" || e == "tmp");
            if ours && std::fs::remove_file(&path).is_ok() {
                dropped += 1;
            }
        }
        Ok(dropped)
    }

    /// This manager's installed set, fetching it only the first time it is asked for.
    ///
    /// The slot's lock is held across the fetch on purpose: two concurrent askers must produce
    /// one subprocess, not two. Different managers hold different slots, so this never
    /// serialises the fan-out across backends.
    /// Returns a shared handle. A caller that needs to own the packages clones it — which is
    /// what [`Queryable::list_installed`] does — and a caller that only wants to look one name up
    /// does not have to.
    pub async fn once<F>(&self, backend: &str, fetch: F) -> Result<Listing>
    where
        F: Future<Output = Result<Vec<Package>>>,
    {
        let slot = self
            .by_backend
            .entry(backend.to_string())
            .or_default()
            .clone();
        let mut slot = slot.lock().await;
        // Read *after* the wait, not before it: the whole question is whether an invalidation
        // happened while this task sat on the mutex.
        let generation = self.generation();
        if let Some((taken_at, cached)) = slot.as_ref() {
            if *taken_at == generation {
                return Ok(cached.clone());
            }
        }
        // The disk layer is consulted inside the slot lock, so two concurrent askers still
        // produce one read rather than two — the same reason the fetch is in here.
        if let Some(on_disk) = self.read_from_disk(backend).await {
            let handle: Listing = Arc::new(on_disk);
            *slot = Some((generation, handle.clone()));
            return Ok(handle);
        }
        // A failure is not cached: a manager that could not answer this time may answer next
        // time, and remembering "it errored" would turn one transient failure into the run's
        // permanent verdict.
        let fresh = fetch.await?;
        self.to_disk(backend, &fresh).await;
        let handle: Listing = Arc::new(fresh);
        // Stamped with the generation the fetch *started* in. An invalidation that landed
        // while the manager was answering makes this answer stale the moment it is stored,
        // which is the correct reading: the listing describes a machine that has since changed.
        *slot = Some((generation, handle.clone()));
        Ok(handle)
    }

    /// Which round of answers is current.
    fn generation(&self) -> u64 {
        self.generation.load(std::sync::atomic::Ordering::Acquire)
    }

    /// This manager's OS-essential set, fetching it only the first time it is asked for.
    ///
    /// Same shape as [`once`](Self::once) and for the same reasons: the slot's lock is held
    /// across the fetch so two concurrent askers produce one subprocess, and the answer
    /// carries the generation it was taken in so an invalidation reaches a handle already
    /// cloned out of the map.
    pub async fn essential_once<F>(&self, backend: &str, fetch: F) -> Result<Essentials>
    where
        F: Future<Output = Result<Vec<String>>>,
    {
        let slot = self
            .essentials
            .entry(backend.to_string())
            .or_default()
            .clone();
        let mut slot = slot.lock().await;
        let generation = self.generation();
        if let Some((taken_at, cached)) = slot.as_ref() {
            if *taken_at == generation {
                return Ok(cached.clone());
            }
        }
        // A failure is not cached, for the reason `once` gives: a manager that could not
        // answer this time may answer next time, and the guard treats "could not ask" as
        // contributing nothing rather than as "nothing is essential".
        let handle: Essentials = Arc::new(fetch.await?);
        *slot = Some((generation, handle.clone()));
        Ok(handle)
    }

    /// [`CommandExecutor::run_output`](crate::core::CommandExecutor::run_output) for a question
    /// whose answer **cannot** depend on what it is asked about, keyed by the whole argv.
    ///
    /// **The caller has to be able to say why, which is why this takes the argv rather than a
    /// name.** A memo is sound only when everything the answer depends on is in the key. The one
    /// caller is a backend's `property_probes` row: `PropertyProbe::resolve` substitutes `{name}`
    /// into the probe's **template** and hands `args` to the subprocess untouched, so the answer
    /// depends on `cmd` + `args` and on nothing else. Twenty packages asked about were twenty
    /// `npm prefix -g` processes to learn one prefix (`PLAN.md` #80); a row that put the package
    /// in its *argv* would get another package's answer back for the rest of the run, and
    /// `a_probe_argv_never_names_the_package` is what says that has not happened.
    ///
    /// **Three properties copied from `once` rather than reinvented here.** The lock is held
    /// *across* the question, because two callers asking the same thing at once must produce one
    /// subprocess — that is the whole point of the lock. A failure is not memoised, for the reason
    /// `once` gives: a manager that could not answer this time may answer next time. And the round
    /// travels in the answer, for `once`'s reason — an entry stamped with the round it *started*
    /// in describes a machine that has since changed. **Kept rather than proved:** dropping the
    /// stamp check leaves every test green, because `forget_all` bumps the generation and clears
    /// the map adjacently, so a question in flight writes into a slot no caller can reach. It
    /// stays because it is the defence `once` has, and two memos should not differ in their one
    /// safety property for the sake of a line that saves a comparison.
    pub(crate) async fn run_output_once_per_run(
        &self,
        exec: &crate::core::CommandExecutor,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<String> {
        let round = self.generation.load(std::sync::atomic::Ordering::Acquire);
        let key = format!("{cmd}\u{1f}{}", args.join("\u{1f}"));
        let slot = self
            .queries
            .entry(key)
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None)))
            .clone();
        let mut held = slot.lock().await;
        if let Some((taken_at, value)) = held.as_ref() {
            if *taken_at == round {
                return Ok(value.clone());
            }
        }
        let value = exec.run_output(cmd, args, sudo).await?;
        *held = Some((round, value.clone()));
        Ok(value)
    }

    /// Forget the listings of the backends that share `manager`'s lock, and keep the rest.
    ///
    /// **The key is the manager, not the backend, and that is `app::stale_lock`'s one table
    /// rather than a second one here.** `pacman` and `yay` are two backends and one lock, so an
    /// install through either has to drop both listings; asking `lock_key(name)` per cached entry
    /// gets that from the table that already answers it, with no registration step and nothing to
    /// forget to wire up.
    ///
    /// **No generation bump, deliberately — and that is the whole difference from `forget_all`.**
    /// The bump is what makes an answer that was already being fetched stale on arrival, and it is
    /// global: bumping it here would invalidate the forty-odd listings this scope exists to keep.
    /// Dropping the map entries is enough for the ones being forgotten, because a task already
    /// holding a cloned `Arc<Slot>` writes into an entry no caller can reach any more.
    ///
    /// **The other two caches go whole, under both scopes.** `essentials` is the set that refuses
    /// removals, so remembering less of it is less safe rather than faster; `queries` has nothing
    /// mapping an argv to the manager that would have to survive.
    pub fn forget_manager(&self, manager: &str) {
        let doomed: Vec<String> = self
            .by_backend
            .iter()
            .filter(|entry| crate::app::stale_lock::lock_key(entry.key()) == manager)
            .map(|entry| entry.key().clone())
            .collect();
        for name in doomed {
            self.by_backend.remove(&name);
            // The disk layer goes with it, or the memo is cleared over a file that survives and
            // the next run re-reads the pre-mutation answer straight off disk. **Only the `.json`:
            // a `.tmp` left by a run killed between the write and the rename is inert** — nothing
            // reads it but `clean-cache` — so sweeping it here would be tidying rather than
            // correctness, and `forget_on_disk` still empties the directory wholesale.
            if self.ttl.is_some() {
                if let Some(path) = Self::cache_file(&name) {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
        self.essentials.clear();
        self.queries.clear();
    }

    /// Forget everything. Called after any mutating command, because that is the only thing
    /// during a run that can change what is installed.
    ///
    /// **The disk layer goes with it.** A memo cleared while a stale file survived would be
    /// re-read from that file on the very next question — so the invalidation that covers one
    /// and not the other is the invalidation that covers neither, which is the shape this repo
    /// has now found in the guard, in the run-scoped memos, and here.
    pub fn forget_all(&self) {
        // The bump is what actually invalidates; the clear is what frees the memory. A task
        // already holding a cloned `Arc<Slot>` never sees the clear, and it is the generation
        // it compares against when it finally wins the lock.
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        self.by_backend.clear();
        self.essentials.clear();
        self.queries.clear();
        if self.ttl.is_some() {
            let _ = Self::forget_on_disk();
        }
    }
}

//! **One `Fixture`.**
//!
//! The same twenty-five lines — a temp root, `shall init`, `run(&["…"])`, collect stdout and
//! stderr — were written **sixteen times**, and they had already drifted three ways:
//!
//! - **`current_dir`** was set by 3 of 16. The other thirteen ran the binary in the *repository
//!   root*, where a stray `shall.txt` is a project-local shell manifest the product reads.
//! - **`HOME`/`USERPROFILE`** was set by 3 of 16. The other thirteen resolved `~` to the machine's
//!   real home directory — so a test that placed a `link:` at `~/.vimrc` was writing to the
//!   developer's actual dotfiles, and passed or failed depending on where the checkout happened
//!   to sit relative to `$HOME`.
//! - **`cfg()`** existed in 11 of 16, and the five without it spelled `root.join("config")` inline.
//!
//! This is the union, and the union is the correct one in every case: a test process that cannot
//! see the developer's home directory or the repository's own working directory is the only kind
//! whose result means anything.
//!
//! **Bespoke helpers stay in their own files.** An inherent `impl` may live in any module of the
//! crate, so a test that needs `dest_mtimes()` or `seed_ledger()` keeps writing
//! `impl Fixture { … }` beside the tests that use it. What moved here is only what was identical.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub struct Fixture {
    pub root: PathBuf,
}

impl Fixture {
    /// A fresh root under `CARGO_TARGET_TMPDIR` with `shall init` already run in it.
    ///
    /// The root is removed first: `CARGO_TARGET_TMPDIR` persists between runs, so a fixture that
    /// only creates is a fixture carrying yesterday's state into today's assertion.
    pub fn new(name: &str) -> Self {
        let f = Self::bare(name);
        let (out, code) = f.run(&["init"]);
        assert_eq!(code, 0, "the fixture's own `init` failed:\n{out}");
        f
    }

    /// The same root, without `init` — for the tests that are *about* an unwritten repository.
    pub fn bare(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    pub fn cfg(&self) -> PathBuf {
        self.root.join("config")
    }

    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }

    /// Write a file into the config repo, creating its parents.
    pub fn write(&self, rel: &str, body: &str) {
        let p = self.cfg().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    /// Run the binary and return everything it said, plus its exit code.
    ///
    /// stdout and stderr are joined because almost every caller wants "what did it say"; the two
    /// that need them apart use [`run_split`](Self::run_split).
    pub fn run(&self, args: &[&str]) -> (String, i32) {
        let (out, err, code) = self.run_split(args);
        (format!("{out}{err}"), code)
    }

    /// The two streams, kept apart — for the tests asserting that a document goes to stdout and
    /// its diagnostics do not.
    pub fn run_split(&self, args: &[&str]) -> (String, String, i32) {
        self.run_split_with(args, None)
    }

    /// `run`, with `dir` prepended to the child's `PATH` — for a test that has to stand in for a
    /// program the verb will drive (`systemctl`), where the alternative is a test that reaches the
    /// developer's session instead of the fixture.
    pub fn run_with_path(&self, dir: &Path, args: &[&str]) -> (String, i32) {
        let (out, err, code) = self.run_split_with(args, Some(dir));
        (format!("{out}{err}"), code)
    }

    fn run_split_with(&self, args: &[&str], path_prefix: Option<&Path>) -> (String, String, i32) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_shall"));
        command
            .args(args)
            // Every one of these four is here because a test that omitted it was measuring the
            // developer's machine rather than the fixture.
            .current_dir(&self.root)
            .env("SHALL_CONFIG_DIR", self.cfg())
            .env("SHALL_DATA_DIR", self.data())
            .env("HOME", &self.root)
            .env("USERPROFILE", &self.root)
            // **`HOME` is not isolation on a host that exports the XDG variables.**
            // `dirs::config_dir()` reads `XDG_CONFIG_HOME` first, and it is set on this machine,
            // so a `schedule:` test provisioned `shall-tidy.timer` into the *developer's*
            // `~/.config/systemd/user` while `HOME` pointed at the fixture (found 2026-11-02 while
            // landing #130, and the units had to be disabled by hand). The fifth entry in this
            // list, for the fifth time the same way.
            .env("XDG_CONFIG_HOME", self.root.join(".config"))
            // A test that blocks on a prompt is a test that hangs CI rather than failing it.
            .stdin(std::process::Stdio::null());
        if let Some(dir) = path_prefix {
            let inherited = std::env::var_os("PATH").unwrap_or_default();
            let mut path = std::ffi::OsString::from(dir);
            path.push(":");
            path.push(inherited);
            command.env("PATH", path);
        }
        let out = command.output().expect("the binary should run");
        (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code().unwrap_or(-1),
        )
    }

    /// `run`, and how long it took in milliseconds.
    pub fn timed(&self, args: &[&str]) -> ((String, i32), u128) {
        let started = Instant::now();
        let answer = self.run(args);
        (answer, started.elapsed().as_millis())
    }

    /// Write the starter module. Two files had this, byte for byte.
    pub fn write_module(&self, body: &str) {
        std::fs::write(self.cfg().join("modules/starter.txt"), body).unwrap();
    }

    /// Put rows into the applied-extras ledger, so a test can start from "this was already
    /// placed" without placing it. Two files had this, byte for byte.
    pub fn seed_ledger(&self, keys: &[&str]) {
        use shall::core::extras_lock::ExtraRecord;
        use shall::core::LockFile;

        let locks = self.cfg().join("locks");
        std::fs::create_dir_all(&locks).unwrap();
        let mut ledger = shall::core::ExtrasLedger::new();
        ledger.record(
            keys.iter()
                .map(|wire| {
                    ExtraRecord::parse(wire)
                        .unwrap_or_else(|e| panic!("`{wire}` is not a ledger row: {e}"))
                })
                .collect(),
        );
        ledger.save(&locks.join("extras.toml")).unwrap();
    }
}

/// A path as a module line may spell it.
///
/// **Forward slashes, always.** The grammar reads `\` as an escape, so a Windows path written
/// raw into a module does not survive the parse. Three files had this function, byte for byte,
/// and each explained it in its own words.
pub fn decl(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// A removal token for a test, obtained the way production obtains one: **by asking the guard**.
///
/// The library's own unit tests use a `#[cfg(test)]` constructor, which an integration test
/// cannot see — it links the release-shaped library. So this asks for real, with a default
/// `Config` (no `protected_packages`, a generous `max_removals`) and an empty registry, which
/// means no manager is consulted and no subprocess runs: the answer is "yes, one removal" in
/// under a millisecond.
///
/// The alternative was a `#[doc(hidden)]` public constructor, and that is the hole `#35` closed:
/// a mint any code can call is a mint the guard does not vouch for, whatever its name.
pub async fn reaped_for_a_test(
    scope: shall::app::sync::guard::GuardScope,
) -> shall::app::sync::guard::Reaped {
    let config = shall::config::Config::default();
    let registry = std::sync::Arc::new(shall::backends::BackendRegistry::new());
    let reaping = shall::app::sync::guard::Reaping::default();
    shall::app::sync::guard::enforce(
        &config,
        &registry,
        &[("test-manager".to_string(), "test-package".to_string())],
        &reaping,
        scope,
    )
    .await
    .expect("the test's own removal is not one the guard refuses")
}

/// The registry's source, which is a **directory** rather than a file.
///
/// `src/backends/registry.rs` was 4,237 lines of which 1,800 were `fn register_*` bodies in
/// declaration order, so "what else is like apt" had no answer but scrolling. It became
/// `registry/mod.rs` plus one module per manager family — and eight scanning gates that read it
/// by path failed in the same second.
///
/// **That is the cost of a split in a repo whose gates are source scans, and it is the argument
/// for having them, not against it.** Every one of those failures was a gate correctly reporting
/// that it could no longer see what it was built to watch. They read this instead, so the next
/// split moves one line rather than eight.
pub fn registry_source() -> String {
    // **Normalised twice, and both are the split's artifacts rather than the code's.**
    //
    // `pub(super) fn register_apt` is the same registrar `fn register_apt` was; the visibility
    // exists only because the parent module now has to call across a file boundary. Six scans
    // keyed on a leading `fn register_` found nothing and reported the *code* broken.
    //
    // And the `#[cfg(test)]` module goes, because these gates read production registrations and
    // a test was never their subject. It mattered the moment `register_generic` stopped being
    // followed by another registrar: the scan that ends one registrar at the next `fn` ran on
    // into the test module, found a `ManagerConfig {` there, and reported a helper as a backend
    // that drops its version pin.
    without_test_modules(&source_of("src/backends/registry"))
        .replace("pub(super) fn ", "fn ")
        .replace("pub fn register_", "fn register_")
}

/// The body of the argv table — the fixture that names every backend's real command line.
///
/// A separate reader because it lives *inside* `#[cfg(test)]`, which [`registry_source`] strips:
/// three gates count the build's backends from these rows, and stripping the test module took
/// the table with it. Two questions, two functions, rather than one function that answers
/// whichever the caller happened to want.
pub fn registry_argv_table() -> String {
    let src = source_of("tests/backends/registry");
    src.split_once("fn argv_cases() -> Vec<ArgvCase> {")
        .expect("the argv table moved or was renamed")
        .1
        .split_once(
            "
    }",
        )
        .expect("the argv table has no end")
        .0
        .to_string()
}

/// Drop every `#[cfg(test)]` module, by matching braces rather than by looking for the next
/// blank line — the distinction between production and test code is a nesting question, and
/// every scan in this repo that got it wrong got it wrong by guessing at a delimiter.
fn without_test_modules(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(at) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let Some(open) = after.find('{') else {
            break;
        };
        let mut depth = 0usize;
        let mut end = None;
        for (i, c) in after[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => rest = &after[e..],
            None => break,
        }
    }
    out.push_str(rest);
    out
}

/// The executor's source, which is three files: the executors themselves, [how a program is
/// found and launched](../../src/core/launch.rs), and [how a child is watched and
/// killed](../../src/core/supervise.rs).
pub fn executor_source() -> String {
    [
        read_source("src/core/executor.rs"),
        read_source("src/core/launch.rs"),
        read_source("src/core/supervise.rs"),
    ]
    .join("\n")
}

fn read_source(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
        .replace("\r\n", "\n")
}

/// Every `.rs` file in a directory, concatenated in name order so the result is the same twice
/// running.
fn source_of(rel: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    assert!(
        !files.is_empty(),
        "{} holds no Rust source; a scan over it would pass over nothing",
        dir.display()
    );
    files.sort();
    files
        .iter()
        .map(|p| {
            std::fs::read_to_string(p)
                .unwrap_or_default()
                .replace("\r\n", "\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **One log collector.** The same twenty-line `Subscriber` had been written twice — once for
/// an async test that needs the guard to outlive an `await`, once for a sync one that wraps a
/// closure — and a third copy was about to be written for the transaction's reporting branch.
/// A collector is not a fixture for one command: it is how any test asks "did this site say
/// what it claims", which is the only way a log-only guard can be pinned at all.
pub fn log_collector() -> (
    std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    tracing::subscriber::DefaultGuard,
) {
    use std::fmt::Write as _;
    use std::sync::{Arc, Mutex};
    use tracing::Subscriber;

    struct Collect(Arc<Mutex<Vec<String>>>);

    impl Subscriber for Collect {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn event(&self, event: &tracing::Event<'_>) {
            let mut visitor = MessageVisitor(Arc::new(Mutex::new(String::new())));
            event.record(&mut visitor);
            let text = std::mem::take(&mut *visitor.0.lock().expect("the log mutex"));
            self.0.lock().expect("the log mutex").push(text);
        }
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}
    }

    struct MessageVisitor(Arc<Mutex<String>>);

    impl tracing::field::Visit for MessageVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let _ = write!(self.0.lock().expect("the log mutex"), "{} ", field.name());
            let _ = write!(self.0.lock().expect("the log mutex"), "{value:?}");
        }
    }

    let seen = Arc::new(Mutex::new(Vec::new()));
    let guard = tracing::subscriber::set_default(Collect(seen.clone()));
    (seen, guard)
}

/// The synchronous shape: run `f` with the collector installed, and hand back what it said.
pub fn captured<F: FnOnce()>(f: F) -> Vec<String> {
    let (seen, guard) = log_collector();
    f();
    // Dropped before the read: the guard is what makes this thread's subscriber the default,
    // and holding it across the return would keep the binding alive past the clone below.
    drop(guard);
    let out = seen.lock().expect("the log mutex").clone();
    out
}

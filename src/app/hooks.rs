//! The three hook dialects (owner ruling, 2026-07-20: all three stay), chosen by a script's
//! first line — a shebang runs it as a process, `#rhai` runs it in-process, anything else is
//! Lua.
//!
//! **They are one feature in three notations, so they get the same things.** All three are
//! handed the same four facts ([`LuaHooks::hook_facts`]), and the `#rhai` arm builds its engine
//! from `core::rhai_stdlib` — the same one `vars.shall` uses, because II.6b already ruled that
//! file *"trusted the same as a hook"* and a hook may not have less than the thing defined by
//! reference to it. It previously had `print` and nothing else, which made the shipped example
//! config's `exec("systemctl enable docker")` a call into an empty room.
//!
//! **None of the three is sandboxed, and none of them ever was.** `Lua::new` loads `os.execute`;
//! a shebang is a process; Rhai now has `sh`. The gate is II.12's ledger — every script hashed,
//! an unapproved or changed one stops the sync, and `-y` cannot skip it.

use crate::config::Config;
use crate::core::hook_lock::{hash_script, hook_id, refusal, HookLedger};
use crate::core::LockFile;
use crate::core::{Error, Result};
use mlua::Lua;
use rhai::{Engine, Scope};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, info};

/// Which language a hook is written in. One place decides, and it hands back the body to run
/// with it — because the marker that chose the dialect is not always part of the script.
pub enum Dialect {
    /// A shebang: written to a file and executed. The shebang line stays — it is what tells the
    /// kernel which interpreter to use, so removing it would break the thing it selected.
    Process,
    Rhai,
    Lua,
}

impl Dialect {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// The dialect, and the script to hand the engine that runs it.
    ///
    /// **`#rhai` is not Rhai.** `#` is a reserved symbol there, so a marker left in place is a
    /// syntax error on line 1 and the script never runs — which is what every `#rhai` hook got.
    /// The marker's line is blanked rather than deleted so that a runtime error still names the
    /// line the author wrote.
    pub fn of(code: &str) -> (Dialect, String) {
        let lead = code.len() - code.trim_start().len();
        let trimmed = &code[lead..];

        if trimmed.starts_with("#!") {
            return (Dialect::Process, code.to_string());
        }
        if trimmed.starts_with("#rhai") {
            let rest_of_script = trimmed.find('\n').map(|i| &trimmed[i..]).unwrap_or("");
            return (
                Dialect::Rhai,
                format!("{}{}", &code[..lead], rest_of_script),
            );
        }
        (Dialect::Lua, code.to_string())
    }
}

pub struct LuaHooks {
    /// Behind an `Arc` so a Rhai evaluation can be handed to the blocking pool. See
    /// [`LuaHooks::run_rhai`].
    rhai_engine: Arc<Engine>,
    pub hooks: HashMap<String, HashMap<String, String>>,
    /// The repo's `locks/` directory, where the hook approval ledger lives (II.12).
    locks_dir: PathBuf,
}

impl LuaHooks {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            rhai_engine: Arc::new(crate::core::rhai_stdlib::engine("hook")),
            hooks: config.hooks.clone(),
            locks_dir: config.layout().locks_dir(),
        })
    }

    /// The supply-chain gate (II.12): before a sync runs any hook, every configured hook must
    /// be approved at its current hash. A new or changed script stops the sync — `-y` cannot
    /// skip this, and only `shall lock` approves. Called with `?` from a place that propagates,
    /// because a swallowed refusal here is no refusal at all.
    ///
    /// Reports every unapproved hook at once, not just the first: a reader fixing their locks
    /// wants the whole list, not a one-at-a-time drip.
    pub fn verify_all_approved(&self) -> Result<()> {
        if self.hooks.is_empty() {
            return Ok(());
        }
        let ledger = HookLedger::load(&HookLedger::path_in(&self.locks_dir))?;
        let mut refusals = Vec::new();
        for (id, script) in self.each_hook() {
            let verdict = ledger.verdict(&id, &hash_script(&script));
            if !verdict.is_approved() {
                refusals.push(refusal(&id, "config", &verdict));
            }
        }
        if refusals.is_empty() {
            return Ok(());
        }
        Err(Error::Refused(format!(
            "refusing to sync: {} hook(s) are not approved (II.12).\n\n{}",
            refusals.len(),
            refusals.join("\n\n")
        )))
    }

    /// Approve every configured hook at its current hash — what `shall lock` does for hooks.
    /// Returns how many approvals were written. This is the only path that writes an approval,
    /// so approval stays a deliberate act.
    pub fn approve_all_hooks(&self) -> Result<usize> {
        let path = HookLedger::path_in(&self.locks_dir);
        HookLedger::update(&path, |ledger| {
            let mut count = 0;
            for (id, script) in self.each_hook() {
                ledger.approve(&id, &hash_script(&script));
                count += 1;
            }
            Ok(count)
        })
    }

    /// Every hook as `(hook_id, script)`. One place builds the identity so enforcement and
    /// approval can never key the ledger differently.
    fn each_hook(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (hook_name, by_package) in &self.hooks {
            for (package, script) in by_package {
                out.push((hook_id(hook_name, package), script.clone()));
            }
        }
        out
    }

    /// Run a `#!` hook, with the interpreter its first line names.
    ///
    /// The interpreter is named on the command line rather than left to the kernel, because
    /// Windows has no shebang mechanism: a script handed straight to `CreateProcess` there fails
    /// with *"not a valid application for this OS platform"* whatever language it is written in,
    /// which reads as "your script is broken" when the platform is what could not read it.
    async fn run_external_polyglot(&self, code: &str, hook: &str, pkg: &str) -> Result<()> {
        debug!("Hooks: Launching Polyglot Bridge for {}/{}", hook, pkg);

        let code_owned = code.to_string();

        let tmp_script = tokio::task::spawn_blocking(move || -> Result<tempfile::TempPath> {
            // The suffix comes from the same place the interpreter does. A shebang naming this
            // platform's own shell lands on `powershell -File`, which refuses any name that does
            // not end in `.ps1`; every other interpreter reads the file and ignores its name.
            let mut tmp = tempfile::Builder::new()
                .suffix(crate::model::script::SCRIPT_SUFFIX)
                .tempfile()
                .map_err(Error::from)?;
            tmp.write_all(code_owned.as_bytes()).map_err(Error::from)?;
            tmp.flush().map_err(Error::from)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(tmp.path())
                    .map_err(Error::from)?
                    .permissions();
                // 0600, where this was 0755. An interpreter named on the command line reads the
                // file, so it never needs the execute bit — and this is the author's script,
                // sitting in a world-readable temp directory for as long as the hook runs.
                perms.set_mode(0o600);
                std::fs::set_permissions(tmp.path(), perms).map_err(Error::from)?;
            }
            // The handle is closed here, keeping only the path (and the delete-on-drop): Windows
            // refuses to let a second process open a file this one still holds.
            Ok(tmp.into_temp_path())
        })
        .await
        .map_err(|e| Error::Other(e.to_string()))??;

        let launch = crate::model::script::launch_for(&tmp_script, code).map_err(|e| {
            Error::Other(format!(
                "{}\nA `#rhai` or Lua hook needs no interpreter and runs wherever Shall does.",
                e
            ))
        })?;

        let mut cmd = Command::new(&launch.program);
        cmd.args(&launch.args);
        for (name, value) in Self::hook_facts(hook, pkg) {
            cmd.env(format!("SHALL_{}", name), value);
        }

        // Supervised: a hook is arbitrary code, and plenty of them run a package manager. Left
        // unowned it outlived the node that fired it — still holding dpkg's lock while the
        // rollback that abandoned it tried to take the same one.
        let out = crate::core::supervise::supervised_output(cmd, "the hook", true)
            .await
            .map_err(|e| Error::Other(format!("Polyglot execution failed: {}", e)))?;

        if !out.status.success() {
            return Err(Error::LuaScript(format!(
                "External hook failed with exit code: {:?}",
                out.status.code()
            )));
        }

        Ok(())
    }

    /// A package-specific hook shadows the `*` catch-all rather than running in addition
    /// to it — a script registered for one package silently disables the global one.
    pub async fn run_hook(&self, hook_name: &str, package_name: &str) -> Result<()> {
        let script = if let Some(category) = self.hooks.get(hook_name) {
            category.get(package_name).or_else(|| category.get("*"))
        } else {
            None
        };

        if let Some(code) = script {
            match Dialect::of(code) {
                (Dialect::Process, body) => {
                    self.run_external_polyglot(&body, hook_name, package_name)
                        .await?
                }
                (Dialect::Rhai, body) => self.run_rhai(&body, hook_name, package_name).await?,
                (Dialect::Lua, body) => self.run_lua(&body, hook_name, package_name).await?,
            }
        }

        Ok(())
    }

    /// Evaluate a `#rhai` hook body, **off the runtime**.
    ///
    /// **It used to be a synchronous `fn` called straight from the async `run_hook`, with no
    /// `block_in_place` and no `spawn_blocking` at all.** Rhai's own `http_get` then does a
    /// blocking `rx.recv()` on a `std::sync::mpsc::sync_channel` while a spawned task performs
    /// the request — so a hook making one HTTP call held the task it was on for the whole
    /// round trip. Hooks run per package around each batch, and the hottest fan-outs are
    /// multiplexed onto a single task, so that stalled every other package in the wave rather
    /// than one of them.
    ///
    /// `block_in_place` would not have fixed it: it moves the runtime's *other tasks* off this
    /// worker and does nothing for futures sharing this task. The evaluation has to leave the
    /// runtime, which is what `off_the_runtime` is for — and it makes the blocking `recv`
    /// inside `http_get` correct rather than merely survivable, because a blocking-pool thread
    /// is a thread that is allowed to sit on something.
    async fn run_rhai(&self, code: &str, hook: &str, pkg: &str) -> Result<()> {
        let engine = self.rhai_engine.clone();
        let code = code.to_string();
        let facts = Self::hook_facts(hook, pkg);
        crate::core::off_the_runtime(move || {
            let mut scope = Scope::new();
            for (name, value) in facts {
                scope.push_constant(name, value);
            }
            engine
                .run_with_scope(&mut scope, &code)
                .map_err(|e| Error::LuaScript(format!("Rhai execution error: {}", e)))
        })
        .await?
    }

    /// What every hook knows about why it is running, whatever dialect it is written in. One
    /// list, because three dialects that each decide for themselves is how the Rhai arm ended up
    /// unable to ask which OS it was on while the Lua and `#!` arms both could.
    pub fn hook_facts(hook: &str, pkg: &str) -> [(&'static str, String); 4] {
        [
            ("PKG_NAME", pkg.to_string()),
            ("HOOK_TYPE", hook.to_string()),
            ("OS", std::env::consts::OS.to_string()),
            ("ARCH", std::env::consts::ARCH.to_string()),
        ]
    }

    /// The Lua interpreter must be constructed INSIDE the blocking closure: `mlua::Lua` is
    /// !Send, so holding one across this boundary will not compile.
    async fn run_lua(&self, code: &str, hook: &str, pkg: &str) -> Result<()> {
        let code_owned = code.to_string();
        let hook_owned = hook.to_string();
        let pkg_owned = pkg.to_string();

        tokio::task::spawn_blocking(move || {
            let lua = Lua::new();
            Self::register_lua_host_functions(&lua)?;

            for (name, value) in Self::hook_facts(&hook_owned, &pkg_owned) {
                lua.globals().set(name, value).map_err(Error::from)?;
            }

            lua.load(&code_owned).exec().map_err(Error::from)?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Other(e.to_string()))??;

        Ok(())
    }

    /// A Lua hook's `print` goes to the log rather than to a stdout the sync is already using.
    /// It was called `setup_lua_sandbox`, which claimed a boundary this does not build: `Lua::new`
    /// loads the standard library, so `os.execute` and `io` are live in every Lua hook. What gates
    /// a hook is II.12's ledger, not the interpreter.
    fn register_lua_host_functions(lua: &Lua) -> Result<()> {
        let globals = lua.globals();
        let print_proxy = lua
            .create_function(|_, args: mlua::MultiValue| {
                let output: Vec<String> = args.iter().map(|v| format!("{:?}", v)).collect();
                info!("[Lua] {}", output.join(" "));
                Ok(())
            })
            .map_err(Error::from)?;

        globals.set("print", print_proxy).map_err(Error::from)?;
        Ok(())
    }

    // SEC7: `render_template` (arbitrary `{{ … }}` evaluated as Lua, with `os`/`io`/`os.execute`
    // left intact in the sandbox) is DELETED. It had no callers — the only `.render_template(`
    // in the tree is the link backend's Tera renderer — so it was dead code, but a loaded gun:
    // wire it to file content and it is ungated RCE. Tera is the live, safe templating path; the
    // gated Lua/Rhai/`#!` *hook* path (approved via the II.12 ledger) is a separate feature and
    // stays. NO-LEGACY: a dead code-exec path is removed, not kept "just in case".

    pub async fn run_before_sync(&self) -> Result<()> {
        self.run_hook("before_sync", "*").await
    }
    pub async fn run_after_sync(&self) -> Result<()> {
        self.run_hook("after_sync", "*").await
    }
}

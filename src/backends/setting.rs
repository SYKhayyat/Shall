//! `setting:SCHEMA/KEY @value=…` — desktop configuration that lives in a settings store
//! rather than a file (X.4).
//!
//! GNOME and KDE keep configuration in dconf and kconfig, not in files `link:` can write, so a
//! tiling-WM config is `link:` and a GNOME toggle is not. The adapter is chosen by the desktop
//! that is running, not by what the user typed — which is why this is a statement, not a
//! backend named after one desktop.
//!
//! **Read before write is what makes it declarative.** A `setting:` that shells out every sync
//! is a command; one that reads the current value and writes only on a difference is a
//! declaration, and only the second belongs in this model.
//!
//! **Removal resets the key to its schema default** (owner ruling, 2026-07-20), rather than
//! restoring whatever value preceded Shall. There is no per-key store of prior values to keep,
//! and "undeclared means the desktop's own default" is the same shape every other statement's
//! removal follows.
//!
//! **An adapter is a row in a table, and the built-ins are rows in it** (K17, ruled
//! 2026-07-23). A closed enum cannot mean *everywhere* — every new desktop would be a Shall
//! release, and the machine running the unusual store is the one that cannot wait for one. So
//! `gsettings` is not special: it is `setting_stores.toml`, shipped, parsed by the loader a
//! user's own row goes through. An adapter mechanism the built-ins bypass is one nobody has
//! tested.

use crate::core::adapter::{self, AdapterRow, Detected};
use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use crate::model::scope::Scope;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, warn};

/// One settings store: how to tell it is here, and the three commands that read, write and
/// reset a key. `{schema}`, `{key}` and `{value}` are substituted into the argv.
#[derive(Debug, Clone, Deserialize)]
pub struct SettingAdapter {
    /// What this store is called, in errors and in `doctor`.
    pub name: String,
    /// The command whose presence on PATH means this machine runs this store.
    pub detect: String,
    /// Restrict the row to one OS (`std::env::consts::OS`). Absent means any.
    #[serde(default)]
    pub os: Option<String>,
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub reset: Vec<String>,
    /// The same three commands for the machine-wide store, when this one has a separate one
    /// (U19). `HKLM` beside `HKCU`; `gsettings` has no counterpart and leaves these unset.
    ///
    /// Absent is not "fall back to the user store" — a `@scope=system` line against a store
    /// with no system commands is refused by name. Writing a user-scope value when the line
    /// said system is the silent wrong answer this model exists to avoid (P7).
    #[serde(default)]
    pub system_read: Vec<String>,
    #[serde(default)]
    pub system_write: Vec<String>,
    #[serde(default)]
    pub system_reset: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SettingStoreFile {
    #[serde(default)]
    pub setting_store: Vec<SettingAdapter>,
}

/// The rows Shall ships. Parsed rather than constructed, so the shipped adapters exercise the
/// same loader a user's row does.
const BUILTIN_STORES: &str = include_str!("setting_stores.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTarget {
    pub account: Option<crate::core::account::Account>,
}

impl StoreTarget {
    pub fn hive(&self) -> String {
        match &self.account {
            None => "HKCU".to_string(),
            Some(account) => match account.sid() {
                Some(sid) => format!("HKEY_USERS\\{sid}"),
                None => String::new(),
            },
        }
    }

    pub fn is_addressable(&self, adapter: &SettingAdapter) -> bool {
        if self.account.is_none() {
            return true;
        }
        if !adapter.addresses_by_hive() {
            return true;
        }
        !self.hive().is_empty()
    }
}

impl SettingAdapter {
    pub fn addresses_by_hive(&self) -> bool {
        [
            &self.read,
            &self.write,
            &self.reset,
            &self.system_read,
            &self.system_write,
        ]
        .iter()
        .any(|cmd| cmd.iter().any(|arg| arg.contains("{hive}")))
    }

    fn fill(
        args: &[String],
        schema: &str,
        key: &str,
        value: &str,
        target: &StoreTarget,
    ) -> Vec<String> {
        adapter::fill(
            args,
            &[
                ("{schema}", schema),
                ("{key}", key),
                ("{value}", value),
                ("{hive}", &target.hive()),
            ],
        )
    }

    /// Split filled argv into the program and its arguments. A row with an empty command is
    /// refused at load, so this never has to invent one.
    fn command(
        args: &[String],
        schema: &str,
        key: &str,
        value: &str,
        target: &StoreTarget,
    ) -> (String, Vec<String>) {
        adapter::program_and_args(Self::fill(args, schema, key, value, target))
            .expect("an empty adapter command was loaded")
    }

    /// Whether this store can be addressed machine-wide at all. A row that leaves the
    /// `system_*` commands unset speaks only for the running user.
    pub fn has_system_scope(&self) -> bool {
        !self.system_read.is_empty()
            && !self.system_write.is_empty()
            && !self.system_reset.is_empty()
    }

    fn argv_for(&self, scope: Scope) -> (&[String], &[String], &[String]) {
        match scope {
            Scope::System if self.has_system_scope() => {
                (&self.system_read, &self.system_write, &self.system_reset)
            }
            _ => (&self.read, &self.write, &self.reset),
        }
    }

    pub fn read_command(&self, scope: Scope, schema: &str, key: &str) -> (String, Vec<String>) {
        self.read_command_for(scope, schema, key, &StoreTarget { account: None })
    }

    pub fn write_command(
        &self,
        scope: Scope,
        schema: &str,
        key: &str,
        value: &str,
    ) -> (String, Vec<String>) {
        self.write_command_for(scope, schema, key, value, &StoreTarget { account: None })
    }

    pub fn reset_command(&self, scope: Scope, schema: &str, key: &str) -> (String, Vec<String>) {
        self.reset_command_for(scope, schema, key, &StoreTarget { account: None })
    }

    pub fn read_command_for(
        &self,
        scope: Scope,
        schema: &str,
        key: &str,
        target: &StoreTarget,
    ) -> (String, Vec<String>) {
        Self::command(self.argv_for(scope).0, schema, key, "", target)
    }

    pub fn write_command_for(
        &self,
        scope: Scope,
        schema: &str,
        key: &str,
        value: &str,
        target: &StoreTarget,
    ) -> (String, Vec<String>) {
        Self::command(self.argv_for(scope).1, schema, key, value, target)
    }

    pub fn reset_command_for(
        &self,
        scope: Scope,
        schema: &str,
        key: &str,
        target: &StoreTarget,
    ) -> (String, Vec<String>) {
        Self::command(self.argv_for(scope).2, schema, key, "", target)
    }
}

impl AdapterRow for SettingAdapter {
    const WHAT: &'static str = "settings adapter";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A row Shall will act on: it can be detected, and all three commands are present. A
    /// store that can be written but not read cannot answer X.4's read-before-write question,
    /// so it is not an adapter — it is a command that runs every sync, which is the thing
    /// `setting:` exists not to be.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.detect.trim().is_empty() {
            return Some("it has no `detect` command");
        }
        if self.read.is_empty() {
            return Some(
                "its `read` command is empty — a store Shall cannot read is one it would write \
                 on every sync",
            );
        }
        if self.write.is_empty() {
            return Some("its `write` command is empty");
        }
        if self.reset.is_empty() {
            return Some("its `reset` command is empty — removing a declaration would do nothing");
        }
        None
    }
}

impl Detected for SettingAdapter {
    fn detect_command(&self) -> &str {
        &self.detect
    }
}

/// Every adapter this machine knows: the shipped rows, then the user's. A user row cannot take
/// a shipped name — that is the same rule custom backends follow, and for the same reason.
pub fn adapters(user_rows: Vec<SettingAdapter>) -> Vec<SettingAdapter> {
    let shipped: SettingStoreFile =
        toml::from_str(BUILTIN_STORES).expect("the shipped setting_stores.toml must parse");
    adapter::merge(shipped.setting_store.into_iter().chain(user_rows))
}

/// Read the user's `[[setting_store]]` rows out of `adapters/settings.toml` (U10).
///
/// Its own file, beside the backend definitions and the bootstrap table — three questions,
/// three files, one folder — but through the *same* approval reader, so an adapter cannot be
/// the one kind of repo-supplied argv that skips II.12.
pub fn user_adapters(cfg: &crate::config::Config) -> Vec<SettingAdapter> {
    let layout = cfg.layout();
    match crate::backends::onboarder::read_approved_definitions(
        &layout.adapter_settings_file(),
        &layout.locks_dir(),
    ) {
        Some(body) => match toml::from_str::<SettingStoreFile>(&body) {
            Ok(f) => f.setting_store,
            Err(e) => {
                warn!(
                    "{}",
                    crate::app::adapters::cannot_use(
                        crate::app::adapters::surface("settings").expect("a declared surface"),
                        e,
                    )
                );
                Vec::new()
            }
        },
        None => Vec::new(),
    }
}

/// Pure: whether the store already holds `want`, given what the read command printed.
///
/// `gsettings` quotes string values (`'prefer-dark'`) and prints bare booleans/enums, and a
/// user writes the bare form. So the comparison holds if the trimmed reading equals the value
/// either bare or quoted — the difference is the store's presentation, not a real one.
pub fn already_set(current: &str, want: &str) -> bool {
    let cur = current.trim();
    if cur == want || cur.trim_matches(['\'', '"']) == want {
        return true;
    }
    // Windows `reg query PATH /v NAME` prints a verbose block whose value line reads
    // `    NAME    REG_SZ    VALUE`. Extract the value after the `REG_*` type token so a
    // registry read-before-write is idempotent — otherwise the value would look unread every
    // sync and be re-written each time (and shown as a pending change). The value may contain
    // spaces (a path), so everything after the type word is kept.
    current.lines().any(|line| {
        line.trim()
            .split_once("REG_")
            .and_then(|(_, after)| after.split_once(char::is_whitespace))
            .is_some_and(|(_, value)| value.trim() == want)
    })
}

pub struct SettingBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    pub adapters: Vec<SettingAdapter>,
}

impl SettingBackendCore {
    pub fn new(executor: CommandExecutor, adapters: Vec<SettingAdapter>) -> Self {
        Self {
            executor,
            name: "setting".to_string(),
            adapters,
        }
    }

    /// The adapter for the store this machine is running: the first row whose `detect`
    /// command is here. `None` makes every `setting:` line an error naming the key — a key
    /// silently unapplied is worse than a refusal, because the whole point is that the file
    /// is the truth (X.4).
    pub fn adapter(&self) -> Option<&SettingAdapter> {
        // Same seam as the init table's, unused by any settings row today: `gsettings` on `PATH`
        // with no session bus behind it is the same wrong answer `systemctl` gave, and a store
        // row can name a liveness path the day someone measures which one it should be.
        adapter::first_present(
            &self.adapters,
            &|c| self.executor.command_exists_sync(c),
            &|p| std::path::Path::new(p).exists(),
        )
    }

    fn split(spec_name: &str) -> Result<(&str, &str)> {
        crate::config::grammar::statement::split_setting(spec_name)
            .ok_or_else(|| Error::Validation(format!("`{}` is not `SCHEMA/KEY`", spec_name)))
    }

    pub fn refuse_system_scope_for_a_named_account(
        &self,
        name: &str,
        scope: Scope,
        user: Option<&str>,
    ) -> Result<()> {
        if scope != Scope::System || user.is_none() {
            return Ok(());
        }
        Err(Error::Validation(format!(
            "`setting:{}` asks for `@scope=system` and for the account `{}` at the same time. \
             A machine-wide value is one value for every account and a named account's value \
             is that account's alone, so there is nothing to write that is both. Drop \
             `@scope=system` to set it for `{}` only, or drop `@user=` to set it for every \
             account.",
            name,
            user.unwrap_or_default(),
            user.unwrap_or_default()
        )))
    }

    /// What scope this line means, refusing rather than pretending when the store cannot do
    /// it (P7: a refusal beats a pretence).
    ///
    /// The default is the store's own — `gsettings` and `HKCU` are per-user because that is
    /// what those stores are — so `@scope=` is written only to override, and **writing the
    /// default is accepted rather than refused as redundant** (owner, 2026-07-24).
    fn scope_of(
        &self,
        adapter: &SettingAdapter,
        written: Option<&str>,
        name: &str,
        user: Option<&str>,
    ) -> Result<Scope> {
        let scope = Scope::resolve(written, Scope::User);
        self.refuse_system_scope_for_a_named_account(name, scope, user)?;
        if scope == Scope::System && !adapter.has_system_scope() {
            return Err(Error::Validation(format!(
                "`setting:{}` asks for scope=system, and the `{}` store Shall found here has \
                 no machine-wide commands. Writing the per-user value instead would apply your \
                 setting to one account while the line says every account. Add `system_read`, \
                 `system_write` and `system_reset` to that `[[setting_store]]` row, or drop \
                 `@scope=system`.",
                name, adapter.name
            )));
        }
        Ok(scope)
    }

    pub fn target_of(
        &self,
        adapter: &SettingAdapter,
        user: Option<&str>,
        name: &str,
    ) -> Result<StoreTarget> {
        let target = StoreTarget {
            account: match user {
                None => None,
                Some(account) => Some(crate::core::account::Account::resolve(account)?),
            },
        };
        if !target.is_addressable(adapter) {
            return Err(Error::Refused(format!(
                "`setting:{}` asks for the account `{}`, and this `{}` store has no address for a \
                 named account on this platform: its commands name a hive or a session that only \
                 exists for the account running Shall. Writing the invoking account's value \
                 instead would apply your setting to one person while the line names another, so \
                 nothing was written. Name the account's store address in this row, or drop \
                 `@user=`.",
                name,
                user.unwrap_or_default(),
                adapter.name
            )));
        }
        Ok(target)
    }

    async fn read_for(&self, target: &StoreTarget, prog: &str, args: &[String]) -> Result<String> {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        match &target.account {
            Some(account) => self.executor.probe_output_as(account, prog, &refs).await,
            None => self.executor.probe_output(prog, &refs).await,
        }
    }

    async fn write_for(&self, target: &StoreTarget, prog: &str, args: &[String]) -> Result<()> {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        match &target.account {
            Some(account) => self
                .executor
                .run_as(account, prog, &refs, false)
                .await
                .map(|_| ()),
            None => self.executor.run(prog, &refs, false).await.map(|_| ()),
        }
    }

    ///
    /// **`None` is "Shall could not ask", and it is never `Some(false)`.** A read fails for
    /// reasons that have nothing to do with the value: a schema `gsettings` has never heard of,
    /// a registry hive this account cannot open, a machine with no settings store on it at all.
    /// Reported as *not in effect*, each of those would make the key look permanently drifted —
    /// `check` red for ever on a key nobody can see, and a write attempted every sync against a
    /// store that just said no. So the read must exit clean to count, which is `probe_output`'s
    /// contract and not `run_output`'s: the latter hands back `Ok("")` for a command that
    /// failed but explained itself, and an empty reading compares unequal to every value there
    /// is.
    ///
    /// The scope refusal is the same shape. A `@scope=system` line against a store with no
    /// machine-wide commands has no readable state at all, and answering from the *user* key
    /// would compare the wrong setting — which is exactly the bug `@scope=` was carried into
    /// the ledger to fix. `scope_of` refuses it by name at apply time; here it is unanswerable.
    pub async fn holds(
        &self,
        spec_name: &str,
        want: &str,
        scope: Option<&str>,
        user: Option<&str>,
    ) -> Option<bool> {
        let (schema, key) = Self::split(spec_name).ok()?;
        let adapter = self.adapter()?;
        let scope = self.scope_of(adapter, scope, spec_name, user).ok()?;
        let target = self.target_of(adapter, user, spec_name).ok()?;
        let (prog, args) = adapter.read_command_for(scope, schema, key, &target);
        let current = self.read_for(&target, &prog, &args).await.ok()?;
        Some(already_set(&current, want))
    }

    fn no_adapter(&self, name: &str) -> Error {
        let known: Vec<&str> = self.adapters.iter().map(|a| a.name.as_str()).collect();
        Error::Validation(format!(
            "`setting:{}` — no settings adapter matches this machine. Shall looked for {}, and \
             found none of them. Add a `[[setting_store]]` row to `adapters/settings.toml` \
             naming the command your store is driven by; a key silently unapplied is worse \
             than an error.",
            name,
            known.join(", ")
        ))
    }
}

#[async_trait]
impl BackendCore for SettingBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        self.adapter().is_some()
    }
    /// The settings stores this OS could have; any one of them is enough.
    fn probes(&self) -> Vec<String> {
        self.adapters
            .iter()
            .filter(|a| a.applies_here())
            .map(|a| a.detect.clone())
            .collect()
    }
    fn needs_root(&self) -> bool {
        // A desktop setting is per-user, written as the user, never with sudo.
        false
    }
}

#[async_trait]
impl MetadataProvider for SettingBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct SettingInstallable {
    pub core: Arc<SettingBackendCore>,
}

#[async_trait]
impl Installable for SettingInstallable {
    async fn install(&self, specs: &[PackageSpec], _sudo: bool) -> Result<()> {
        for spec in specs {
            let (schema, key) = SettingBackendCore::split(&spec.name)?;
            let want = spec.options.one("value").ok_or_else(|| {
                Error::Validation(format!("`setting:{}` has no value", spec.name))
            })?;

            let Some(adapter) = self.core.adapter() else {
                return Err(self.core.no_adapter(&spec.name));
            };
            let scope = self.core.scope_of(
                adapter,
                spec.options.one("scope"),
                &spec.name,
                spec.options.one("user"),
            )?;
            let target = self
                .core
                .target_of(adapter, spec.options.one("user"), &spec.name)?;

            // Read before write: only touch the store when it does not already hold `want`,
            // so a settled sync runs no command at all. Through the same probe `plan` and
            if self
                .core
                .holds(
                    &spec.name,
                    want,
                    spec.options.one("scope"),
                    spec.options.one("user"),
                )
                .await
                == Some(true)
            {
                continue;
            }

            let (prog, args) = adapter.write_command_for(scope, schema, key, want, &target);
            self.core.write_for(&target, &prog, &args).await?;
            info!("Setting {}/{} = {}", schema, key, want);
        }
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        _sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        for name in names {
            // **A removal resets the key where the declaration put it, which means it has to
            // know where that was.** The scope rides the extras-ledger key — `setting:x@scope=
            // system` — because it is the only place it can: by the time the teardown runs, the
            // line that carried `@scope=` is gone. Before this the removal always reset
            // `Scope::User`, so deleting a system-scoped line reset the *user* key and left the
            // machine-wide value in place, reporting success.
            let (bare, user) = crate::core::extras_lock::split_removal(
                crate::config::grammar::ResourceKind::Setting,
                name,
            )?;
            let (bare, scope) = split_scope(&bare);
            let (schema, key) = SettingBackendCore::split(bare)?;
            self.core
                .refuse_system_scope_for_a_named_account(bare, scope, user.as_deref())?;
            // A store with no adapter never held the value, so there is nothing to reset and
            // nothing to fail on.
            let Some(adapter) = self.core.adapter() else {
                continue;
            };
            // A store with no machine-wide commands cannot have written a machine-wide value,
            // so there is nothing there to reset — and resetting the user key instead is the
            // bug this whole path is about. `scope_of` refuses that combination at apply time.
            if scope == Scope::System && !adapter.has_system_scope() {
                continue;
            }
            let target = self.core.target_of(adapter, user.as_deref(), bare)?;
            let (prog, args) = adapter.reset_command_for(scope, schema, key, &target);
            self.core.write_for(&target, &prog, &args).await?;
            info!(
                "Setting {}/{} reset to its default ({} scope, {})",
                schema,
                key,
                scope.as_str(),
                target
                    .account
                    .as_ref()
                    .map_or("this account".to_string(), |a| format!(
                        "account `{}`",
                        a.name()
                    ))
            );
        }
        Ok(())
    }
}

/// Split a `SCHEMA/KEY@scope=system` ledger subject into the key and the scope it was written
/// at. No suffix means the store's own default, which is what an unscoped declaration means and
/// what every ledger row written before this said.
///
/// An unrecognised scope spelling reads as the default rather than refusing: this runs during a
/// teardown, and a row Shall cannot fully parse is still a row naming something it put there.
/// Resetting the default is what it did before the scope was carried at all.
fn split_scope(subject: &str) -> (&str, Scope) {
    match subject.split_once("@scope=") {
        Some((bare, scope)) => (bare, Scope::parse(scope).unwrap_or(Scope::User)),
        None => (subject, Scope::User),
    }
}

pub struct SettingQueryable {
    pub core: Arc<SettingBackendCore>,
}

#[async_trait]
impl Queryable for SettingQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        // A setting is not software with an inventory: Shall knows the keys it declares, not
        // every key the store holds. Nothing to enumerate.
        Ok(vec![])
    }
    async fn list_manual(&self) -> Result<Vec<Package>> {
        Ok(vec![])
    }
    async fn info(&self, _name: &str) -> Result<Option<Package>> {
        Ok(None)
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    cfg: &crate::config::Config,
) {
    let core = Arc::new(SettingBackendCore::new(
        exec.clone(),
        adapters(user_adapters(cfg)),
    ));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(SettingInstallable { core: core.clone() }))
            .with_queryable(Arc::new(SettingQueryable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gsettings() -> SettingAdapter {
        adapters(vec![])
            .into_iter()
            .find(|a| a.name == "gsettings")
            .expect("the shipped table must carry gsettings")
    }

    /// The shipped rows go through the loader, not around it. If this fails, the built-in
    /// adapters have stopped being the thing a user's row is.
    #[test]
    fn the_shipped_table_parses_and_carries_gsettings() {
        let all = adapters(vec![]);
        let names: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
        // The shipped rows are unfiltered here (OS selection happens in `adapter()`), so both
        // built-ins are present regardless of host.
        assert!(names.contains(&"gsettings"), "{:?}", names);
        assert!(names.contains(&"windows-registry"), "{:?}", names);
        let gs = all.iter().find(|a| a.name == "gsettings").unwrap();
        assert_eq!(gs.detect, "gsettings");
        assert_eq!(gs.os.as_deref(), Some("linux"));
    }

    #[test]
    fn gsettings_reads_writes_and_resets_a_key() {
        let a = gsettings();
        let (p, args) = a.read_command(Scope::User, "org.gnome.x", "k");
        assert_eq!(p, "gsettings");
        assert_eq!(args, vec!["get", "org.gnome.x", "k"]);

        let (_, args) = a.write_command(Scope::User, "org.gnome.x", "k", "v");
        assert_eq!(args, vec!["set", "org.gnome.x", "k", "v"]);

        let (_, args) = a.reset_command(Scope::User, "org.gnome.x", "k");
        assert_eq!(args, vec!["reset", "org.gnome.x", "k"]);
    }

    /// `gsettings` dispatches on argv[1] by hand, so a `--` is read as the command name and
    /// the call fails before it reaches the schema.
    #[test]
    fn gsettings_deliberately_gets_no_option_terminator() {
        assert!(!crate::core::argv::terminates_options("gsettings"));
        let a = gsettings();
        for (_, args) in [
            a.read_command(Scope::User, "org.gnome.x", "k"),
            a.write_command(Scope::User, "org.gnome.x", "k", "v"),
            a.reset_command(Scope::User, "org.gnome.x", "k"),
        ] {
            assert!(!args.iter().any(|x| x == "--"), "{:?}", args);
        }
    }

    fn row(name: &str, detect: &str) -> SettingAdapter {
        SettingAdapter {
            name: name.into(),
            detect: detect.into(),
            os: None,
            read: vec![
                detect.into(),
                "read".into(),
                "{schema}".into(),
                "{key}".into(),
            ],
            write: vec![
                detect.into(),
                "write".into(),
                "{schema}".into(),
                "{key}".into(),
                "{value}".into(),
            ],
            reset: vec![
                detect.into(),
                "reset".into(),
                "{schema}".into(),
                "{key}".into(),
            ],
            system_read: vec![],
            system_write: vec![],
            system_reset: vec![],
        }
    }

    /// K17: a store Shall has never heard of is a row, not a release. This is the whole point
    /// of the ruling — the adapter below is driven from a definition, through the same code
    /// path the shipped one uses.
    #[test]
    fn a_store_with_no_compiled_in_support_is_driven_from_a_row() {
        let all = adapters(vec![row("kde", "kwriteconfig6")]);
        let kde = all
            .iter()
            .find(|a| a.name == "kde")
            .expect("the user row loaded");
        let (prog, args) = kde.write_command(
            Scope::User,
            "kdeglobals/General",
            "ColorScheme",
            "BreezeDark",
        );
        assert_eq!(prog, "kwriteconfig6");
        assert_eq!(
            args,
            vec!["write", "kdeglobals/General", "ColorScheme", "BreezeDark"]
        );
    }

    #[test]
    fn a_user_row_cannot_take_a_shipped_name() {
        let all = adapters(vec![row("gsettings", "impostor")]);
        let g = all.iter().find(|a| a.name == "gsettings").unwrap();
        assert_eq!(g.detect, "gsettings", "a user row shadowed the shipped one");
        // The impostor is rejected, so only the shipped rows remain.
        assert_eq!(all.len(), 2);
    }

    /// A row that cannot be read is not an adapter: X.4's read-before-write is what makes
    /// `setting:` a declaration rather than a command that runs every sync.
    #[test]
    fn a_row_missing_a_command_is_refused_rather_than_half_used() {
        let mut unreadable = row("half", "halfctl");
        unreadable.read = vec![];
        assert!(!adapters(vec![unreadable]).iter().any(|a| a.name == "half"));

        let mut unresettable = row("half", "halfctl");
        unresettable.reset = vec![];
        assert!(!adapters(vec![unresettable])
            .iter()
            .any(|a| a.name == "half"));

        let mut nameless = row("", "halfctl");
        nameless.name = String::new();
        // Only the shipped rows remain once the nameless one is refused.
        assert_eq!(adapters(vec![nameless]).len(), 2);
    }

    #[test]
    fn a_row_for_another_os_is_not_this_machines_store() {
        let mut elsewhere = row("elsewhere", "cmd");
        elsewhere.os = Some("plan9".into());
        assert!(!elsewhere.applies_here());
        let mut here = row("here", "cmd");
        here.os = Some(std::env::consts::OS.to_string());
        assert!(here.applies_here());
    }

    fn registry_like() -> SettingAdapter {
        let mut r = row("winreg", "reg");
        r.system_read = vec!["reg".into(), "query".into(), "HKLM/{schema}".into()];
        r.system_write = vec!["reg".into(), "add".into(), "HKLM/{schema}".into()];
        r.system_reset = vec!["reg".into(), "delete".into(), "HKLM/{schema}".into()];
        r
    }

    /// U19: a store with both scopes runs different commands for each. Without this the two
    /// scopes would be a label on identical behaviour.
    #[test]
    fn a_store_with_both_scopes_runs_different_commands() {
        let a = registry_like();
        assert!(a.has_system_scope());
        let (_, user) = a.write_command(Scope::User, "Software/X", "k", "v");
        let (_, system) = a.write_command(Scope::System, "Software/X", "k", "v");
        assert_ne!(user, system, "system scope reused the per-user command");
        assert!(system.iter().any(|x| x.contains("HKLM")), "{:?}", system);
    }

    /// A store with no machine-wide commands says so rather than quietly writing the per-user
    /// value — the silent wrong answer P7 refuses.
    #[test]
    fn a_store_without_system_scope_reports_it() {
        assert!(!gsettings().has_system_scope());
        // ...and asking for system scope falls back to nothing: the caller must refuse, which
        // `scope_of` does. The argv itself never silently becomes the user one for a caller
        // that checked first.
        let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
        let err = core
            .scope_of(&gsettings(), Some("system"), "org.gnome.x/k", None)
            .expect_err("system scope on a user-only store must be refused")
            .to_string();
        assert!(err.contains("gsettings"), "{}", err);
        assert!(err.contains("one account"), "{}", err);
    }

    /// The owner's clarification: writing the scope that is already the default is accepted,
    /// not refused as redundant. A config may state what it would also get for free.
    #[test]
    fn writing_the_default_scope_is_accepted() {
        let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
        let g = gsettings();
        assert_eq!(
            core.scope_of(&g, Some("user"), "org.gnome.x/k", None)
                .unwrap(),
            Scope::User
        );
        // And omitting it means the same thing.
        assert_eq!(
            core.scope_of(&g, None, "org.gnome.x/k", None).unwrap(),
            Scope::User
        );
    }

    #[test]
    fn a_machine_wide_scope_named_at_one_account_is_refused() {
        let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
        let registry = registry_like();
        let name = "org.gnome.x/theme";

        for (scope, user) in [
            (Scope::System, Some("alice")),
            (Scope::System, Some("someone-else")),
        ] {
            let err = core
                .refuse_system_scope_for_a_named_account(name, scope, user)
                .expect_err("a machine-wide value cannot also be one account's")
                .to_string();
            assert!(err.contains(name), "{err}");
            assert!(err.contains(user.unwrap()), "{err}");
            assert!(err.contains("@scope=system"), "{err}");
            assert!(err.contains("@user="), "{err}");
        }

        for (scope, user) in [
            (Scope::System, None),
            (Scope::User, Some("alice")),
            (Scope::User, None),
        ] {
            assert!(
                core.refuse_system_scope_for_a_named_account(name, scope, user)
                    .is_ok(),
                "({scope:?}, {user:?}) is not a contradiction"
            );
        }

        let err = core
            .scope_of(&registry, Some("system"), name, Some("alice"))
            .expect_err("the apply must not reach the store with this line")
            .to_string();
        assert!(err.contains("nothing to write that is both"), "{err}");
    }

    #[tokio::test]
    async fn a_teardown_of_a_contradictory_row_is_refused_before_it_resets_anything() {
        let core = Arc::new(SettingBackendCore::new(
            CommandExecutor::new(false, false),
            adapters(vec![registry_like()]),
        ));
        let installable = SettingInstallable { core: core.clone() };
        let err = installable
            .remove(
                &["org.gnome.x/theme@scope=system@user=alice".to_string()],
                false,
                crate::app::sync::guard::reaped_for_a_unit_test(
                    crate::app::sync::guard::GuardScope::Sync,
                    "a unit test of the teardown itself",
                ),
            )
            .await
            .expect_err("a row that was never written must not be reset");
        let message = err.to_string();
        assert!(
            message.contains("nothing to write that is both"),
            "{message}"
        );
        assert!(message.contains("alice"), "{message}");
    }

    #[test]
    fn a_quoted_reading_equals_the_bare_value() {
        // `gsettings get` prints `'prefer-dark'`; the user wrote `prefer-dark`. Same value.
        assert!(already_set("'prefer-dark'", "prefer-dark"));
        assert!(already_set("prefer-dark\n", "prefer-dark"));
        assert!(already_set("true", "true"));
        // A registry read prints `"Dark"`; the same rule covers it.
        assert!(already_set("\"Dark\"", "Dark"));
    }

    #[test]
    fn a_different_value_is_not_already_set() {
        assert!(!already_set("'prefer-light'", "prefer-dark"));
        assert!(!already_set("false", "true"));
    }

    #[test]
    fn a_reg_query_block_is_read_as_its_value() {
        // 7e: `reg query PATH /v NAME` prints a verbose block. The value after the REG_* type
        // token is what Shall compares, so a matching value is not re-written every sync.
        let out = "\r\nHKEY_CURRENT_USER\\Software\\App\r\n    Theme    REG_SZ    Dark\r\n\r\n";
        assert!(already_set(out, "Dark"));
        assert!(!already_set(out, "Light"));
        // A value with spaces (a path) survives — everything after the type word is the value.
        let path = "    Wallpaper    REG_SZ    C:\\Users\\me\\a b.jpg\r\n";
        assert!(already_set(path, "C:\\Users\\me\\a b.jpg"));
    }

    /// A store driven by a real child process, so the read is parsed rather than handed over.
    /// `{schema}` and `{key}` are substituted and then ignored — what this fixture controls is
    /// the *reading*, which is the half `holds` has to get right.
    fn store_reading(reading: &str) -> SettingBackendCore {
        #[cfg(windows)]
        let read = vec!["cmd".into(), "/C".into(), format!("echo {}", reading)];
        #[cfg(not(windows))]
        let read = vec![
            "sh".into(),
            "-c".into(),
            format!("printf '%s' '{}'", reading),
        ];
        let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
        a.read = read;
        SettingBackendCore::new(CommandExecutor::new(false, false), vec![a])
    }

    /// The J2 fix, in one assertion: the reporting half asks the store the same question the
    /// installer has always asked it. Before this, every `setting:` line was *unverifiable* —
    /// which places — so `check` and `plan` named a settled key as work on every run.
    #[tokio::test]
    async fn a_key_the_store_already_holds_reads_back_as_in_effect() {
        let core = store_reading("prefer-dark");
        assert_eq!(
            core.holds(
                "org.gnome.desktop.interface/color-scheme",
                "prefer-dark",
                None,
                None
            )
            .await,
            Some(true)
        );
        assert_eq!(
            core.holds(
                "org.gnome.desktop.interface/color-scheme",
                "prefer-light",
                None,
                None
            )
            .await,
            Some(false),
            "a different value is drift, and drift places"
        );
    }

    /// **The caveat that ships with the ruling.** A read fails for reasons that are not "the
    /// value differs": a schema `gsettings` has never heard of, a hive this account cannot
    /// open. Read as *not in effect*, each would make `check` permanently red on a key nobody
    /// can see and make every sync rewrite it. Only a clean read counts.
    #[tokio::test]
    async fn a_read_the_store_refuses_is_unanswerable_not_absent() {
        let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
        a.read = if cfg!(windows) {
            vec!["cmd".into(), "/C".into(), "exit 1".into()]
        } else {
            vec!["sh".into(), "-c".into(), "exit 1".into()]
        };
        let core = SettingBackendCore::new(CommandExecutor::new(false, false), vec![a]);
        assert_eq!(
            core.holds("org.gnome.x/k", "anything", None, None).await,
            None,
            "a refused read must not be reported as a value that differs"
        );
    }

    /// The other two ways the question is unanswerable, each for its own reason and neither of
    /// them "the value differs". A machine with no store cannot be asked; a name that is not
    /// `SCHEMA/KEY` addresses nothing to ask about.
    #[tokio::test]
    async fn a_machine_with_no_store_and_a_name_with_no_key_are_both_unanswerable() {
        let none = SettingBackendCore::new(CommandExecutor::new(false, false), vec![]);
        assert_eq!(none.holds("org.gnome.x/k", "v", None, None).await, None);

        let core = store_reading("prefer-dark");
        assert_eq!(core.holds("dark", "prefer-dark", None, None).await, None);
    }

    /// U19's rule, on the reading side: a `@scope=system` line against a store with no
    /// machine-wide commands is unanswerable, not answered from the per-user key. Reading one
    /// scope and writing the other is the bug `@scope=` was carried into the ledger to fix, and
    /// it would arrive here as a confident `Some`.
    #[tokio::test]
    async fn system_scope_on_a_user_only_store_is_unanswerable() {
        let core = store_reading("prefer-dark");
        assert_eq!(
            core.holds("org.gnome.x/k", "prefer-dark", Some("system"), None)
                .await,
            None
        );
        // ...and the same store answers the scope it does have.
        assert_eq!(
            core.holds("org.gnome.x/k", "prefer-dark", Some("user"), None)
                .await,
            Some(true)
        );
    }

    #[test]
    fn the_registry_row_addresses_a_hive_rather_than_a_literal_hkcu() {
        let reg = adapters(vec![])
            .into_iter()
            .find(|a| a.name == "windows-registry")
            .expect("the registry row must ship");
        for argv in [&reg.read, &reg.write, &reg.reset] {
            assert!(
                argv.iter().any(|a| a.contains("{hive}")),
                "{argv:?} does not name a hive, so it can only ever address the invoking account"
            );
            assert!(
                !argv.iter().any(|a| a == "HKCU\\{schema}"),
                "{argv:?} still addresses a literal HKCU"
            );
        }
        let (_, args) = reg.read_command_for(
            Scope::User,
            "Software\\X",
            "k",
            &StoreTarget { account: None },
        );
        assert!(
            args.iter().any(|a| a.contains("HKCU")),
            "an unscoped registry read is not HKCU: {args:?}"
        );
    }

    #[test]
    fn a_store_that_cannot_address_a_named_account_refuses_before_writing() {
        let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
        let account = crate::core::account::Account::current().unwrap();
        let target = StoreTarget {
            account: Some(account.clone()),
        };
        let addressable = target.is_addressable(&registry());
        if addressable {
            let hive = target.hive();
            assert!(hive.starts_with("HKEY_USERS"), "{hive}");
            assert!(hive.contains(&account.sid().expect("a SID on this platform")));
            assert!(core
                .target_of(&registry(), Some(account.name()), "X/k")
                .is_ok());
        } else {
            let err = core
                .target_of(&registry(), Some(account.name()), "X/k")
                .expect_err("a store with no address for a named account must refuse")
                .to_string();
            assert!(err.contains(account.name()), "{err}");
            assert!(err.contains("nothing was written"), "{err}");
            assert!(err.contains("windows-registry"), "{err}");
        }
    }

    fn registry() -> SettingAdapter {
        adapters(vec![])
            .into_iter()
            .find(|a| a.name == "windows-registry")
            .expect("the registry row must ship")
    }

    #[tokio::test]
    async fn a_read_is_asked_in_the_account_the_line_named() {
        let account = crate::core::account::Account::current().unwrap();
        let read = if cfg!(windows) {
            vec!["cmd".into(), "/C".into(), "echo %USERNAME%".into()]
        } else {
            vec!["sh".into(), "-c".into(), "printf '%s' \"$USER\"".into()]
        };
        let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
        a.read = read;
        let core = SettingBackendCore::new(CommandExecutor::new(false, false), vec![a]);

        let mine = core
            .holds("org.gnome.x/k", account.name(), None, Some(account.name()))
            .await;
        assert_eq!(
            mine,
            Some(true),
            "a read for the account the line named did not answer from that account"
        );
        assert_eq!(
            core.holds("org.gnome.x/k", account.name(), None, None)
                .await,
            Some(true),
            "the unscoped read and the self-named read disagree about the same account"
        );
        assert_eq!(
            core.holds("org.gnome.x/k", "not-the-value", None, Some(account.name()))
                .await,
            Some(false),
            "a read that succeeded reported no comparison rather than a difference"
        );
    }

    #[test]
    fn a_command_driven_store_is_addressed_by_where_it_runs_and_a_registry_one_by_its_argv() {
        let gs = gsettings();
        let unscoped = StoreTarget { account: None };
        let here = StoreTarget {
            account: Some(crate::core::account::Account::current().unwrap()),
        };
        for (label, target) in [("unscoped", &unscoped), ("this account", &here)] {
            let (_, user) = gs.read_command_for(Scope::User, "org.gnome.x", "k", target);
            let (_, system) = gs.read_command_for(Scope::System, "org.gnome.x", "k", target);
            assert_eq!(user, vec!["get", "org.gnome.x", "k"], "{label}");
            assert_eq!(system, user, "{label}: gsettings has no machine-wide store");
        }

        let reg = registry();
        let (_, plain) = reg.read_command_for(Scope::User, "Software\\X", "k", &unscoped);
        let (_, named) = reg.read_command_for(Scope::User, "Software\\X", "k", &here);
        assert!(
            plain.iter().any(|a| a.contains("HKCU")),
            "an unscoped registry read is not HKCU: {plain:?}"
        );
        if here.is_addressable(&reg) {
            assert_ne!(
                plain, named,
                "a registry read for a named account is byte-identical to the unscoped one, so \
                 it would address the invoking account's hive"
            );
            assert!(
                named.iter().any(|a| a.contains("HKEY_USERS")),
                "a named registry read does not address that account's own hive: {named:?}"
            );
        }
    }

    /// The refusal names what Shall looked for, so a machine running an unlisted store learns
    /// what to write a row about rather than only that it failed.
    #[test]
    fn the_refusal_names_the_stores_it_looked_for() {
        let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
        let msg = core
            .no_adapter("org.gnome.desktop.interface/color-scheme")
            .to_string();
        assert!(msg.contains("gsettings"), "{}", msg);
        assert!(msg.contains("setting_store"), "{}", msg);
    }
}

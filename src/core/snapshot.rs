use crate::config::Config;
use crate::core::adapter::{self, AdapterRow, Detected};
use crate::core::{CommandExecutor, Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command as StdCommand;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: String,
    pub description: String,
    pub backend: String,
}

impl Snapshot {
    pub fn parse_time(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    }

    /// Recover a snapshot's creation time from the timestamp Shall embeds in the id it
    /// generates (S2). `list()` cannot get this from btrfs/zfs — their creation-time flags and
    /// output formats vary by version — but every id Shall makes carries the time in a fixed
    /// shape:
    ///
    /// - btrfs: `shall_pre_<label>_<YYYYMMDDHHMMSS>`
    /// - zfs:   `<dataset>@shall_<YYYYMMDD_HHMMSS>`
    ///
    /// The digits are local wall-clock (that is how `create()` formats them), so they are read
    /// back as local time. Returns `None` for an id in neither shape — e.g. a snapshot Shall
    /// did not create — so the caller can fall back rather than trust a wrong time.
    ///
    /// This is the fix for the bug where `list()` stamped every snapshot with `Utc::now()`, so
    /// each read as zero seconds old and age-based retention (`max_age_days`, `keep_days`) could
    /// never fire — a retention policy that silently keeps everything (P3).
    pub fn time_from_id(id: &str) -> Option<DateTime<Utc>> {
        // zfs first: the part after the last `@shall_`, formatted `%Y%m%d_%H%M%S`.
        if let Some(rest) = id.rsplit_once("@shall_") {
            if let Ok(naive) = NaiveDateTime::parse_from_str(rest.1.trim(), "%Y%m%d_%H%M%S") {
                return local_naive_to_utc(naive);
            }
        }
        // btrfs: the trailing `_<14 digits>` group.
        if let Some(tail) = id.rsplit('_').next() {
            if tail.len() == 14 && tail.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(naive) = NaiveDateTime::parse_from_str(tail, "%Y%m%d%H%M%S") {
                    return local_naive_to_utc(naive);
                }
            }
        }
        None
    }

    /// The rfc3339 string for the [`Snapshot::time_from_id`] of `id`, or `None` if the id
    /// carries no recognizable time. Used by `list()` to fill the `timestamp` field.
    pub fn timestamp_from_id(id: &str) -> Option<String> {
        Self::time_from_id(id).map(|t| t.to_rfc3339())
    }

    /// Whether Shall created this snapshot — the ownership test retention uses so it never
    /// reclaims a restore point the user made by hand (S3).
    ///
    /// The marker lands in different fields per provider: btrfs/zfs put `shall_` in the **id**
    /// (`shall_pre_…`, `…@shall_…`), while Windows System Restore forces the id to a bare
    /// `SequenceNumber` and carries `Shall:` in the **description**. Checking only the id — the
    /// old bug — meant nothing Shall created on Windows was ever pruned.
    ///
    /// **Anchored, not substring, and only in the shapes `create()` writes.** A substring test
    /// claims any snapshot whose name happens to contain the letters — "Marshall weekly" — and
    /// retention then deletes it.
    pub fn is_shall_owned(&self) -> bool {
        if self.id.starts_with("shall_pre_") || self.id.contains("@shall_") {
            return true;
        }
        self.description.to_lowercase().starts_with("shall:")
    }
}

/// Interpret a naive datetime as local wall-clock (how snapshot ids are formatted) and convert
/// to UTC. Ambiguous local times (a DST fall-back hour) resolve to the earlier instant, which
/// for a retention age is close enough and never panics.
fn local_naive_to_utc(naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Why Shall took a snapshot. There are exactly these four, and they are the only text that
/// reaches the Windows provider's PowerShell interpolation — a `&str` there would put a future
/// `--label` flag one hop from an elevated shell (SEC5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotLabel {
    PreSync,
    PreUpgrade,
    PurgeUndeclared,
    PreCanary,
    PreRebuild,
    /// Taken before `bisect` starts restoring, and restored when it stops.
    ///
    /// **`bisect` used to leave the machine wherever the binary search happened to end.** It is
    /// a diagnostic — "which change broke this?" — and it answered by rearranging your installed
    /// software into an arbitrary historical state and returning `Ok(())` without a word. Not
    /// even the culprit's state: whichever candidate the last iteration probed.
    PreBisect,
}

impl SnapshotLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            SnapshotLabel::PreSync => "pre_sync",
            SnapshotLabel::PreUpgrade => "pre_upgrade",
            SnapshotLabel::PurgeUndeclared => "purge-undeclared",
            SnapshotLabel::PreCanary => "pre_canary",
            SnapshotLabel::PreRebuild => "pre_rebuild",
            SnapshotLabel::PreBisect => "pre_bisect",
        }
    }
}

impl std::fmt::Display for SnapshotLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Taking a snapshot and putting one back are different capabilities, and a provider can
/// have the first without the second. `btrfs subvolume snapshot SRC /` exits 0 and creates a
/// nested subvolume; nothing is rolled back. Everything that offers an undo asks this first,
/// so the offer is not made where it cannot be kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreCapability {
    /// The running system is put back by `restore`.
    Live,
    /// The snapshot is real and restorable, but not from a running system. `how` says what
    /// the person at the machine has to do instead. Owned because a config-driven provider
    /// (U27) supplies its own sentence, and a built-in supplies a `&'static` one via `.into()`.
    NotFromRunningSystem { how: String },
}

impl RestoreCapability {
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }

    /// The one sentence `doctor` and the pre-change notice both print, so the two cannot
    /// come to disagree about what this machine can do.
    pub fn describe(&self, provider: &str) -> String {
        match self {
            Self::Live => format!("{}: snapshots can be taken and restored.", provider),
            Self::NotFromRunningSystem { how } => format!(
                "{}: snapshots can be taken but NOT restored from a running system — {}",
                provider, how
            ),
        }
    }
}

#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn is_available(&self) -> bool;
    async fn create(&self, label: SnapshotLabel) -> Result<Snapshot>;
    async fn list(&self) -> Result<Vec<Snapshot>>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn restore(&self, id: &str) -> Result<()>;
    fn restore_capability(&self) -> RestoreCapability;
}

/// A Windows restore point is a `SequenceNumber`. The delete/restore cmdlets interpolate it
/// unquoted and run elevated, so it becomes a `u32` here or it does not reach them at all —
/// there is no quoting to get right for a number (SEC5). This is the typed gate the Windows
/// snapshot row substitutes `{id}` through (V.82): a row can name the cmdlet, but the id it fills
/// is validated as a `u32` first, so a free-text template with an id spliced into a shell is
/// unrepresentable.
pub fn windows_sequence_number(id: &str) -> Result<u32> {
    id.trim().parse::<u32>().map_err(|_| {
        Error::Validation(format!(
            "`{}` is not a Windows restore point — an id is a SequenceNumber, a plain number.",
            id
        ))
    })
}

/// A snapshot provider described entirely in `adapters/snapshot.toml` (U27) — the same "rows,
/// not Rust" move the backend, firewall, settings and init layers already made. A filesystem
/// with create/list/delete/restore-shaped commands becomes a provider with no source change.
///
/// **The one rule that keeps this from being the V.60 footgun: a capability the row does not
/// declare, it does not have.** `restores_running_system` defaults to `false`, so a provider is
/// create-only unless the file *says* it can put a running machine back — and saying so is the
/// line a reviewer sees in the diff. A row that omits it can snapshot and can refuse a rollback;
/// it can never run a command that "restores" and rolls nothing back.
#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotProviderDef {
    pub name: String,
    /// Restrict to one OS (`std::env::consts::OS`). Absent means any.
    #[serde(default)]
    pub os: Option<String>,
    /// The command whose presence means this provider can act on this machine.
    pub detect: String,
    /// A path that must also exist for the provider to be available — btrfs is only usable if
    /// its snapshot subvolume is mounted, which the `detect` binary existing does not prove.
    #[serde(default)]
    pub detect_path: Option<String>,
    /// What `{source}` expands to (a dataset, a volume group, a subvolume path). Optional.
    #[serde(default)]
    pub source: String,
    /// Placeholders: `{id}`, `{label}`, `{source}`.
    pub create: Vec<String>,
    pub list: Vec<String>,
    /// Placeholders: `{id}`.
    pub delete: Vec<String>,
    /// Placeholders: `{id}`. Empty means this provider cannot restore at all.
    #[serde(default)]
    pub restore: Vec<String>,
    /// The safe default (U27/V.60): a provider is create-only unless the file names this true.
    #[serde(default)]
    pub restores_running_system: bool,
    /// A regex whose first capture group is a snapshot id on each `list` line. An optional
    /// second group, when present, is the snapshot's description — which is where Windows carries
    /// the `Shall:` ownership marker (btrfs/zfs carry it in the id instead).
    pub list_pattern: String,
    /// The sentence shown when a create-only provider is asked to restore. A default is supplied
    /// when the row omits it, so the refusal is never blank.
    #[serde(default)]
    pub restore_how: Option<String>,
    /// How Shall builds the id it names a snapshot with, when it is Shall that names it (btrfs,
    /// zfs, lvm). Placeholders: `{label}`, `{source}`, `{ts}` (`%Y%m%d%H%M%S`, 14 contiguous
    /// digits so [`Snapshot::time_from_id`] can read the age back) and `{ts_}` (`%Y%m%d_%H%M%S`,
    /// the zfs shape). Absent and with no `create_id_pattern`, the id defaults to
    /// `shall_<label>_<%Y%m%d_%H%M%S>`.
    #[serde(default)]
    pub id_template: Option<String>,
    /// When the *tool* names the snapshot (timeshift, apfs), this regex reads the id back from
    /// the create command's stdout — its first capture group is the id. Mutually exclusive with
    /// `id_template`: either Shall names it or the tool does.
    #[serde(default)]
    pub create_id_pattern: Option<String>,
    /// Whether `list` must run elevated (timeshift's `--list` needs root; btrfs/zfs do not).
    #[serde(default)]
    pub list_needs_root: bool,
    /// Windows System Restore is not argv — it is elevated PowerShell cmdlets. When set, each
    /// command is one PowerShell line run via `powershell -Command`, and `{id}` is substituted
    /// **only** after [`windows_sequence_number`] parses it as a `u32`, so nothing but a number
    /// reaches the elevated shell (SEC5/V.82). This is the one shape a snapshot row carries
    /// beyond plain argv.
    #[serde(default)]
    pub powershell: bool,
}

impl SnapshotProviderDef {
    /// Whether a declared row can actually put a running system back. `restores_running_system`
    /// alone is not enough — a row that claims it but gives no `restore` command still cannot,
    /// and claiming `Live` there is exactly the V.60 lie.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn is_live(&self) -> bool {
        self.restores_running_system && !self.restore.is_empty()
    }
}

impl AdapterRow for SnapshotProviderDef {
    const WHAT: &'static str = "snapshot provider";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A row Shall will drive, or why it will not. It must be able to create, list and delete;
    /// restore is the capability that is allowed to be absent, and its absence is the safe state.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.detect.trim().is_empty() {
            return Some("it has no `detect` command");
        }
        if self.create.is_empty() {
            return Some("it cannot create a snapshot");
        }
        if self.list.is_empty() {
            return Some("it cannot list snapshots, so retention could never reap them");
        }
        if self.delete.is_empty() {
            return Some("it cannot delete a snapshot, so retention could never reap them");
        }
        if self.list_pattern.trim().is_empty() {
            return Some("it has no `list_pattern`, so a listed line has no id");
        }
        None
    }
}

impl Detected for SnapshotProviderDef {
    fn detect_command(&self) -> &str {
        &self.detect
    }
}

pub struct ConfigSnapshotProvider {
    pub executor: CommandExecutor,
    pub def: SnapshotProviderDef,
}

impl ConfigSnapshotProvider {
    fn fill(cmd: &[String], id: &str, label: &str, source: &str) -> Vec<String> {
        adapter::fill(
            cmd,
            &[("{id}", id), ("{label}", label), ("{source}", source)],
        )
    }

    /// The id Shall generates for a provider it names itself (btrfs/zfs/lvm). The `shall_` marker
    /// is what ownership (S3) and retention key on, so a config provider can never make a user's
    /// own snapshots look like Shall's.
    fn generated_id(&self, label: SnapshotLabel) -> String {
        match &self.def.id_template {
            Some(t) => t
                .replace("{label}", label.as_str())
                .replace("{ts}", &Local::now().format("%Y%m%d%H%M%S").to_string())
                .replace("{ts_}", &Local::now().format("%Y%m%d_%H%M%S").to_string())
                .replace("{source}", &self.def.source),
            None => format!(
                "shall_{}_{}",
                label.as_str(),
                Local::now().format("%Y%m%d_%H%M%S")
            ),
        }
    }

    async fn run(&self, cmd: Vec<String>) -> Result<()> {
        let (prog, args) = cmd
            .split_first()
            .ok_or_else(|| Error::Snapshot("a snapshot command is empty".into()))?;
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.executor.run(prog, &refs, true).await.map(|_| ())
    }

    /// Fill one PowerShell line, substituting `{id}` as a validated `u32` (V.82) — the typed slot
    /// that makes a Windows snapshot row safe by construction. A non-numeric id is refused before
    /// it can reach the elevated shell.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn fill_ps(&self, template: &str, id: Option<&str>, label: &str) -> Result<String> {
        let mut s = template.to_string();
        if s.contains("{id}") {
            let id = id.ok_or_else(|| Error::Snapshot("a snapshot id is required".into()))?;
            let seq = windows_sequence_number(id)?;
            s = s.replace("{id}", &seq.to_string());
        }
        Ok(s.replace("{label}", label)
            .replace("{source}", &self.def.source))
    }

    async fn run_ps(&self, command: &str, elevated: bool) -> Result<String> {
        // `-NoProfile` because a user's PowerShell profile can add hundreds of milliseconds to
        // seconds to every invocation, and none of it is work Shall asked for; `-NonInteractive`
        // because this output is captured, so a prompt here would be a question asked into a
        // pipe nobody is showing. `psresource.rs` and `executor.rs` have passed `-NoProfile`
        // all along — this was the third of three.
        self.executor
            .run_output(
                "powershell",
                &["-NoProfile", "-NonInteractive", "-Command", command],
                elevated,
            )
            .await
    }

    fn first_command<'a>(&self, cmd: &'a [String], what: &str) -> Result<&'a str> {
        cmd.first()
            .map(String::as_str)
            .ok_or_else(|| Error::Snapshot(format!("the {} command is empty", what)))
    }
}

#[async_trait]
impl SnapshotProvider for ConfigSnapshotProvider {
    fn name(&self) -> &str {
        &self.def.name
    }

    async fn is_available(&self) -> bool {
        if !self.def.applies_here() {
            return false;
        }
        if !self.executor.command_exists(&self.def.detect).await {
            return false;
        }
        // btrfs's binary can be present on a machine whose root is not btrfs; the snapshot
        // subvolume existing is what proves the provider can act.
        if let Some(p) = &self.def.detect_path {
            if !Path::new(p).exists() {
                return false;
            }
        }
        true
    }

    async fn create(&self, label: SnapshotLabel) -> Result<Snapshot> {
        let id = if self.def.powershell {
            // Windows' create cmdlet does not expose the SequenceNumber directly. The provider
            // row therefore appends a query for the newest restore point and declares a pattern
            // for that output. Returning a synthetic id here made the object handed to delete
            // and restore unusable: those cmdlets require the real sequence number.
            let template = self.def.create.clone();
            let line = self.fill_ps(
                self.first_command(&template, "create")?,
                None,
                label.as_str(),
            )?;
            info!("{}: creating snapshot ({})", self.def.name, label);
            let out = self.run_ps(&line, true).await?;
            let pattern = self.def.create_id_pattern.as_ref().ok_or_else(|| {
                Error::Snapshot(format!(
                    "{} creates restore points but has no create_id_pattern",
                    self.def.name
                ))
            })?;
            let re = crate::utils::regex_cache::compiled(pattern)
                .map_err(|e| Error::Snapshot(format!("bad create_id_pattern: {}", e)))?;
            re.captures_iter(&out)
                .filter_map(|c| c.get(1))
                .last()
                .map(|m| m.as_str().trim().to_string())
                .ok_or_else(|| {
                    Error::Snapshot(format!(
                        "{} created a restore point but did not return its SequenceNumber",
                        self.def.name
                    ))
                })?
        } else if let Some(pattern) = &self.def.create_id_pattern {
            // The tool names the snapshot; read the id back from its output.
            let cmd = Self::fill(&self.def.create, "", label.as_str(), &self.def.source);
            let (prog, args) = cmd
                .split_first()
                .ok_or_else(|| Error::Snapshot("the create command is empty".into()))?;
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let out = self.executor.run_output(prog, &refs, true).await?;
            let re = crate::utils::regex_cache::compiled(pattern)
                .map_err(|e| Error::Snapshot(format!("bad create_id_pattern: {}", e)))?;
            re.captures(&out)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_else(|| self.generated_id(label))
        } else {
            let id = self.generated_id(label);
            let cmd = Self::fill(&self.def.create, &id, label.as_str(), &self.def.source);
            info!("{}: creating snapshot {}", self.def.name, id);
            self.run(cmd).await?;
            id
        };
        Ok(Snapshot {
            id,
            timestamp: Utc::now().to_rfc3339(),
            description: label.to_string(),
            backend: self.def.name.clone(),
        })
    }

    async fn list(&self) -> Result<Vec<Snapshot>> {
        let out = if self.def.powershell {
            let template = self.def.list.clone();
            self.run_ps(self.first_command(&template, "list")?, false)
                .await?
        } else {
            let (prog, args) = self
                .def
                .list
                .split_first()
                .ok_or_else(|| Error::Snapshot("a snapshot list command is empty".into()))?;
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.executor
                .run_output(prog, &refs, self.def.list_needs_root)
                .await?
        };
        let re = crate::utils::regex_cache::compiled(&self.def.list_pattern)
            .map_err(|e| Error::Snapshot(format!("bad list_pattern: {}", e)))?;
        let mut snaps = Vec::new();
        for line in out.lines() {
            let Some(caps) = re.captures(line) else {
                continue;
            };
            let Some(m) = caps.get(1) else { continue };
            let id = m.as_str().to_string();
            // Group 2, when the pattern captures it, is the description — where Windows keeps its
            // `Shall:` ownership marker. Otherwise the provider name stands in.
            let description = caps
                .get(2)
                .map(|d| d.as_str().to_string())
                .unwrap_or_else(|| self.def.name.clone());
            snaps.push(Snapshot {
                timestamp: Snapshot::timestamp_from_id(&id)
                    .unwrap_or_else(|| Utc::now().to_rfc3339()),
                id,
                description,
                backend: self.def.name.clone(),
            });
        }
        Ok(snaps)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        if self.def.powershell {
            let template = self.def.delete.clone();
            let line = self.fill_ps(self.first_command(&template, "delete")?, Some(id), "")?;
            self.run_ps(&line, true).await.map(|_| ())
        } else {
            let cmd = Self::fill(&self.def.delete, id, "", &self.def.source);
            self.run(cmd).await
        }
    }

    async fn restore(&self, id: &str) -> Result<()> {
        if !self.def.is_live() {
            // The V.60 refusal, config-driven: a provider that did not declare live restore does
            // not run a "restore" that might roll nothing back. It says so and leaves the
            // snapshot intact.
            return Err(Error::Snapshot(format!(
                "{}: this provider cannot roll a running system back to {}. {}",
                self.def.name,
                id,
                self.def
                    .restore_how
                    .clone()
                    .unwrap_or_else(|| "The snapshot is intact; restore it by hand.".into())
            )));
        }
        if self.def.powershell {
            let template = self.def.restore.clone();
            let line = self.fill_ps(self.first_command(&template, "restore")?, Some(id), "")?;
            self.run_ps(&line, true).await.map(|_| ())
        } else {
            let cmd = Self::fill(&self.def.restore, id, "", &self.def.source);
            self.run(cmd).await
        }
    }

    fn restore_capability(&self) -> RestoreCapability {
        if self.def.is_live() {
            RestoreCapability::Live
        } else {
            RestoreCapability::NotFromRunningSystem {
                how: self.def.restore_how.clone().unwrap_or_else(|| {
                    "this provider was not declared able to restore a running system".into()
                }),
            }
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SnapshotProviderFile {
    #[serde(default)]
    pub snapshot: Vec<SnapshotProviderDef>,
}

/// The config-driven providers this repo carries, if `adapters/snapshot.toml` is approved
/// through the one II.12 ledger every adapter file goes through. An unapproved or unparseable
/// file yields none, loudly — never a silent partial safety net.
fn config_snapshot_defs(config: &Config) -> Vec<SnapshotProviderDef> {
    let layout = config.layout();
    let path = layout.adapter_snapshot_file();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            warn!("could not read adapters/snapshot.toml: {}", e);
            return Vec::new();
        }
    };
    if let Some(refusal) =
        crate::core::hook_lock::adapter_refusal(&path, &content, &layout.locks_dir())
    {
        tracing::error!("{}", refusal);
        return Vec::new();
    }
    match toml::from_str::<SnapshotProviderFile>(&content) {
        Ok(f) => adapter::usable(f.snapshot),
        Err(e) => {
            warn!(
                "{}",
                crate::app::adapters::cannot_use(
                    crate::app::adapters::surface("snapshot").expect("a declared surface"),
                    e,
                )
            );
            Vec::new()
        }
    }
}

/// The snapshot providers Shall ships (U27, Option A). Compiled into the binary, so — unlike the
/// user's `adapters/snapshot.toml` — they are not read through the hook ledger: a first-party
/// asset cannot be tampered with by a pulled config, and gating it would leave a fresh machine
/// with no safety net until `shall lock` ran.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub const BUILTIN_SNAPSHOT_DEFS: &str = include_str!("snapshot_builtins.toml");

/// The auto-detected zfs root dataset, when `zfs_dataset` is not configured. Empty when zfs is
/// absent or the query fails — which drops the row rather than shipping a source-less one.
fn detect_zfs_root() -> String {
    // **Asked before it is spawned.** This ran on every process that built the provider table,
    // on every machine, whatever `priority` said — a `zfs` probe on a container where nothing
    // else is, which is how it was noticed. Spawning a program to find out it is not installed
    // is the expensive way to ask a question `resolve_program` answers from a memo, and on a
    // host with no `zfs` it is a process launch that can only fail.
    if crate::core::launch::resolve_program("zfs").is_none() {
        return String::new();
    }
    let mut cmd = StdCommand::new("zfs");
    cmd.args(["list", "-H", "-o", "name,mountpoint", "-r", "/"])
        .stdin(std::process::Stdio::null());
    // **The dataset whose MOUNTPOINT is `/`, not whichever line sorts first.** `zfs list -r`
    // orders by name, so on a pool where the root dataset is not alphabetically first the old
    // first-line read snapshotted — and declared Live — some unrelated descendant. `-H` gives
    // tab-separated fields; mountpoint is the second.
    crate::core::blocking::command_output(&mut cmd)
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .filter_map(|l| {
                    let mut f = l.split('\t');
                    let name = f.next()?.trim();
                    let mp = f.next()?.trim();
                    Some((name, mp))
                })
                .find(|(_, mp)| *mp == "/")
                .map(|(name, _)| name.to_string())
        })
        .unwrap_or_default()
}

/// The built-in provider rows, with btrfs/zfs's machine-specific `source`/`detect_path` filled
/// from config. A row whose source is still unresolved (zfs with no dataset) is dropped, because
/// a provider that cannot name what to snapshot is a create that fails, not a safety net.
fn builtin_snapshot_defs(config: &Config) -> Vec<SnapshotProviderDef> {
    let mut file: SnapshotProviderFile = toml::from_str(BUILTIN_SNAPSHOT_DEFS)
        .expect("built-in snapshot defs are compiled in and must parse");
    for def in &mut file.snapshot {
        match def.name.as_str() {
            "btrfs" => {
                def.source = config.btrfs_path.clone();
                def.detect_path = Some(config.btrfs_path.clone());
            }
            "zfs" => {
                def.source = config
                    .zfs_dataset
                    .clone()
                    .filter(|d| !d.trim().is_empty())
                    .unwrap_or_else(detect_zfs_root);
            }
            _ => {}
        }
    }
    // Through the same floor a user's row clears, which is the whole of K17/U1: an adapter
    // mechanism the built-ins bypass is one nobody has tested. The shipped rows all pass it —
    // `the_builtin_snapshot_defs_hold_their_invariants` says so directly — so this costs
    // nothing today and catches the row that stops passing it tomorrow.
    adapter::usable(
        file.snapshot
            .into_iter()
            .filter(|d| !(d.name == "zfs" && d.source.trim().is_empty())),
    )
}

pub struct SnapshotManager {
    provider: Option<Box<dyn SnapshotProvider>>,
}

impl SnapshotManager {
    pub fn with_provider(provider: Box<dyn SnapshotProvider>) -> Self {
        Self {
            provider: Some(provider),
        }
    }

    pub async fn new(executor: CommandExecutor, config: &Config) -> Self {
        let mut providers: Vec<Box<dyn SnapshotProvider>> = Vec::new();

        // The built-in providers are rows through the same loader a user row goes through (U27,
        // V.82) — btrfs, timeshift, apfs, windows, then zfs (last, so btrfs wins on a machine
        // with both). There is no hardcoded provider `Vec` any more.
        for def in builtin_snapshot_defs(config) {
            providers.push(Box::new(ConfigSnapshotProvider {
                executor: executor.clone(),
                def,
            }));
        }

        // User-declared providers register LAST (U27), so a `adapters/snapshot.toml` row can
        // never shadow a built-in — the `custom_backends.toml` rule applied to the safety layer.
        for def in config_snapshot_defs(config) {
            providers.push(Box::new(ConfigSnapshotProvider {
                executor: executor.clone(),
                def,
            }));
        }

        let active = Self::choose(providers, &config.snapshot_priority).await;
        Self { provider: active }
    }

    /// The active provider (U28). When a `snapshot_priority` is declared, the first *available*
    /// provider named in it wins — chosen by the user's declared order, not by registration
    /// order and not by capability-guessing. A provider named in the list but absent from the
    /// machine is skipped; a name in the list that matches no provider is ignored. With no list,
    /// the first available in registration order wins (built-ins first), which is the historical
    /// behaviour untouched.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub async fn choose(
        providers: Vec<Box<dyn SnapshotProvider>>,
        priority: &[String],
    ) -> Option<Box<dyn SnapshotProvider>> {
        // Which are actually usable on this machine, in registration order.
        //
        // Probed concurrently: each `is_available` is a real `command_exists` plus a path
        // check, and this sits in front of the snapshot a sync takes. `buffered` preserves the
        // registration order the rest of this function depends on, which `buffer_unordered`
        // would not.
        //
        // The width is the number of providers, not a setting: this is the built-in
        // registration list, a handful of entries fixed at compile time, so "all of them at
        // once" is a statement about the list rather than a cap somebody might want to move.
        use futures::stream::StreamExt;
        let width = providers.len().max(1);
        let mut available: Vec<Box<dyn SnapshotProvider>> = futures::stream::iter(providers)
            .map(|p| async move { p.is_available().await.then_some(p) })
            .buffered(width)
            .filter_map(|p| async move { p })
            .collect()
            .await;
        if priority.is_empty() {
            return available.into_iter().next();
        }
        for want in priority {
            if let Some(pos) = available
                .iter()
                .position(|p| p.name().eq_ignore_ascii_case(want))
            {
                return Some(available.swap_remove(pos));
            }
        }
        // A declared priority that names nothing present: fall back rather than leave the machine
        // with no safety net it could have had.
        available.into_iter().next()
    }

    pub async fn auto_snapshot(&self, label: SnapshotLabel) -> Result<Option<Snapshot>> {
        if let Some(ref p) = self.provider {
            Ok(Some(p.create(label).await?))
        } else {
            Ok(None)
        }
    }

    /// True when an active snapshot provider is available (so rollback is possible).
    /// Used by `canary`/`bisect`/policy `require_snapshot` to fail fast when there is no
    /// safety net rather than performing an unrecoverable change.
    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    pub fn provider_name(&self) -> Option<&str> {
        self.provider.as_ref().map(|p| p.name())
    }

    /// What this machine's provider can do, or `None` when it takes no snapshots at all.
    pub fn restore_capability(&self) -> Option<RestoreCapability> {
        self.provider.as_ref().map(|p| p.restore_capability())
    }

    /// The one place a snapshot is put back. `undo` and every other recovery path calls
    /// this, so a provider that refuses refuses everywhere.
    pub async fn restore(&self, id: &str) -> Result<()> {
        let p = self.provider.as_ref().ok_or_else(|| {
            Error::Snapshot("this machine takes no snapshots, so there is none to restore".into())
        })?;
        p.restore(id).await
    }

    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        if let Some(ref p) = self.provider {
            p.list().await
        } else {
            Ok(vec![])
        }
    }

    /// Only ever deletes snapshots whose id contains "shall", so retention cannot reap a
    /// user's or another tool's snapshots. Inactive policy or no provider = no-op.
    /// Apply the configured retention policy now, at this moment's clock.
    ///
    /// **One derivation of "which snapshots does this run keep", for two callers.** `shall
    /// snapshot prune` and the pass `sync` runs after a successful transaction each built the
    /// same three arguments themselves, so the policy, the clock and the meaning of
    /// `--dry-run` were decided twice and could drift. `force` is `prune --force`: prune for
    /// real on a run that is otherwise a preview.
    pub async fn prune_by_policy(&self, config: &Config, force: bool) -> Result<Vec<String>> {
        let dry_run = !force && config.dry_run;
        self.prune_with_policy(&config.snapshot_retention(), Utc::now(), dry_run)
            .await
    }

    pub async fn prune_with_policy(
        &self,
        policy: &crate::core::RetentionPolicy,
        now: DateTime<Utc>,
        dry_run: bool,
    ) -> Result<Vec<String>> {
        let p = match &self.provider {
            Some(p) => p,
            None => return Ok(vec![]),
        };
        if !policy.prunes() {
            return Ok(vec![]);
        }
        let list: Vec<Snapshot> = p
            .list()
            .await?
            .into_iter()
            .filter(|s| s.is_shall_owned())
            .collect();
        let items: Vec<crate::core::RetentionItem> = list
            .iter()
            .map(|s| {
                crate::core::RetentionItem::new(s.id.clone(), s.parse_time().unwrap_or(now))
                    .labelled(s.description.clone())
            })
            .collect();
        let doomed = policy.select_deletions(&items, now);
        // The return value is what the caller prints as "pruned N", so it carries only the
        // ids whose delete actually succeeded — a snapshot still on disk must never be
        // counted as reaped.
        let mut pruned = Vec::new();
        let mut failed = Vec::new();
        for id in &doomed {
            if dry_run {
                crate::would!("retention would prune {}", id);
                pruned.push(id.clone());
            } else {
                match p.delete(id).await {
                    Ok(()) => pruned.push(id.clone()),
                    Err(e) => failed.push(format!("{} ({})", id, e)),
                }
            }
        }
        if !failed.is_empty() {
            warn!(
                "retention could not delete {} snapshot(s), still on disk: {}",
                failed.len(),
                failed.join(", ")
            );
        }
        Ok(pruned)
    }

    pub async fn restore_snapshot(&self, id: &str) -> Result<()> {
        if let Some(ref p) = self.provider {
            p.restore(id).await
        } else {
            Err(Error::Snapshot("No active provider".into()))
        }
    }
}

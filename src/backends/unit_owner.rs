//! Which package owns a file, answered by the manager that owns it (`#211`).
//!
//! **Rows, not Rust.** The four managers that can answer are rows in `unit_owners.toml`, parsed by
//! the same approved loader every other adapter file goes through — a mechanism the built-ins
//! bypass is one nobody has tested, and the reason `builtin_backends.toml` exists at all.
//!
//! The question is asked when `adopt` decides what to write beside a service line: a unit found
//! running is a unit somebody installed, and the package that owns its file is the one thing that
//! puts it back on the next machine. Where the answer is not knowable — no manager claims the
//! path, the manager is not installed, the file is not a package's — nothing is written. A wrong
//! name beside a line is worse than no name: a reader cannot check it without doing the work
//! Shall was supposed to do.
use serde::Deserialize;

use crate::core::adapter::{merge, AdapterRow, Detected};
use crate::core::executor::CommandExecutor;

const BUILTIN_OWNERS: &str = include_str!("unit_owners.toml");

/// One manager's answer to "which package owns this file".
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitOwner {
    pub name: String,
    /// The platform this row is for. Absent means every platform the row is visible on.
    #[serde(default)]
    pub os: Option<String>,
    /// The command whose presence on `PATH` means this manager can answer here.
    detect: String,
    /// argv with `{path}` substituted.
    pub owns: Vec<String>,
    /// The reader in `src/parsers/named.rs` (`owning` direction) that reads the answer.
    pub reads: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitOwnerFile {
    #[serde(default)]
    owner: Vec<UnitOwner>,
}

impl AdapterRow for UnitOwner {
    const WHAT: &'static str = "unit owner";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A row Shall will act on: it can be detected, and it knows how to ask and how to read.
    /// A row with argv and no reader would answer a machine's question with nothing — the same
    /// silence `every_row_can_read_what_it_asks_for` refuses for a listing.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.detect.trim().is_empty() {
            return Some("it has no `detect` command");
        }
        if self.owns.is_empty() {
            return Some("its `owns` command is empty");
        }
        if self.reads.trim().is_empty() {
            return Some("it names no `reads` reader, so its answer cannot be understood");
        }
        None
    }
}

impl Detected for UnitOwner {
    fn detect_command(&self) -> &str {
        &self.detect
    }
}

/// Every row this machine knows: the shipped rows, then the user's. A user row cannot take a
/// shipped name — the same rule custom backends follow, and for the same reason.
pub fn owners(user_rows: Vec<UnitOwner>) -> Vec<UnitOwner> {
    let shipped: UnitOwnerFile =
        toml::from_str(BUILTIN_OWNERS).expect("the shipped unit_owners.toml must parse");
    merge(shipped.owner.into_iter().chain(user_rows))
}

/// The user's `[[owner]]` rows out of `adapters/unit-owners.toml`.
///
/// Through the same approval reader every repo-supplied adapter goes through (`II.12`), so a row
/// that answers "which package owns this file" cannot be the one kind that skips the lock.
pub fn user_owners(cfg: &crate::config::Config) -> Vec<UnitOwner> {
    let layout = cfg.layout();
    match crate::backends::onboarder::read_approved_definitions(
        &layout.adapter_unit_owners_file(),
        &layout.locks_dir(),
    ) {
        Some(body) => match toml::from_str::<UnitOwnerFile>(&body) {
            Ok(f) => f.owner,
            Err(e) => {
                tracing::warn!("adapters/unit-owners.toml is not a row file: {}", e);
                Vec::new()
            }
        },
        None => Vec::new(),
    }
}

/// The package that owns `path`, asked of the first manager on this machine that can answer.
///
/// `priority` is the caller's own order — the registry's, so a question about a file is answered
/// by the manager this machine installs with rather than by whichever row was parsed first.
/// Nothing here fails: an unanswered path is the ordinary case (a hand-written unit, a file no
/// package owns, a manager that is not installed), and the caller writes nothing when it is.
pub async fn owning_package(
    executor: &CommandExecutor,
    rows: &[UnitOwner],
    priority: &[String],
    path: &str,
) -> Option<String> {
    let order = |row: &UnitOwner| {
        priority
            .iter()
            .position(|p| p == &row.name)
            .unwrap_or(priority.len())
    };
    let mut usable: Vec<&UnitOwner> = rows
        .iter()
        .filter(|row| row.applies_here() && row.unusable().is_none())
        .collect();
    usable.sort_by_key(|row| order(row));
    for row in usable {
        let Some(reader) = crate::parsers::named::owning(&row.reads) else {
            continue;
        };
        let argv: Vec<&str> = row
            .owns
            .iter()
            .map(|a| if a == "{path}" { path } else { a.as_str() })
            .collect();
        let Some((prog, args)) = argv.split_first() else {
            continue;
        };
        let Ok(out) = executor.run_output(prog, args, false).await else {
            // A manager that refuses the question has not answered it, and the next row is
            // still worth asking.
            continue;
        };
        if let Some(package) = reader(&out, path).into_iter().next() {
            return Some(package);
        }
    }
    None
}

/// Write each adopted unit's owning package into the candidate's own properties.
///
/// **Nothing is asked when no manager on this machine can answer.** On a NixOS box, a Mac and a
/// Windows host this costs one parse and no commands, rather than one `systemctl show` per
/// adopted service for an answer that could never come.
///
/// **The unit's own file, not a guessed directory.** `systemctl show -p FragmentPath` is the only
/// answer to "where is this unit" that does not assume `/lib/systemd/system`, and a unit that is a
/// hand-written drop-in has no package at all — in which case nothing is written, because a
/// comment naming the wrong package is a lie the reader cannot check.
pub async fn attach_owners(
    executor: &CommandExecutor,
    cfg: &crate::config::Config,
    packages: &mut [crate::core::Package],
    priority: &[String],
) {
    if !packages.iter().any(|p| p.backend == "service") {
        return;
    }
    let rows = owners(user_owners(cfg));
    let can_answer = rows.iter().any(|row| {
        row.applies_here()
            && row.unusable().is_none()
            && executor.command_exists_sync(Detected::detect_command(row))
    });
    if !can_answer {
        return;
    }
    for pkg in packages.iter_mut() {
        if pkg.backend != "service" {
            continue;
        }
        let Some(path) = unit_file_of(executor, &pkg.name).await else {
            continue;
        };
        if let Some(owner) = owning_package(executor, &rows, priority, &path).await {
            pkg.properties.insert("owned_by".to_string(), owner);
        }
    }
}

/// The file a unit was loaded from, if the init will say.
async fn unit_file_of(executor: &CommandExecutor, name: &str) -> Option<String> {
    let shown = executor
        .run_output(
            "systemctl",
            &["--no-pager", "show", "-p", "FragmentPath", "--", name],
            false,
        )
        .await
        .ok()?;
    let path = shown
        .lines()
        .find_map(|line| line.strip_prefix("FragmentPath="))?
        .trim();
    (!path.is_empty()).then(|| path.to_string())
}

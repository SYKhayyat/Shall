//! The core every generic backend shares: what binary it runs, how it is probed, and what
//! its manager's output means.

use super::*;

pub struct GenericBackendCore {
    pub name: String,
    pub executor: CommandExecutor,
    pub config: ManagerConfig,
    pub parser: Arc<dyn OutputParser>,
}

impl GenericBackendCore {
    /// The program this backend runs. `name` is the prefix a line is written with, and for
    /// every built-in the two are the same word — but a user-defined noun (`firewall:`) runs
    /// something else (`ufw`), so a command position must ask for this and never for `name`
    /// (XIII.12). `list_binary`/`search_binary`/`enumerate_binary` are narrower overrides and
    /// fall back to this, not to the name.
    pub fn binary(&self) -> &str {
        self.config.binary.as_deref().unwrap_or(&self.name)
    }

    /// What this backend's mutations are exclusive over.
    ///
    /// **The manager, never the program.** OpenBSD installs with `pkg_add` and removes with
    /// `pkg_delete`, and keying on the program gave those two verbs two different locks over one
    /// package database — so a `shall` installing and a `shall` removing could hold both at
    /// once. Every hand-written backend already named its manager here; the shared machinery was
    /// the one place that named the binary.
    /// What this backend takes an exclusive lock on — the **manager**, not the backend.
    ///
    /// Usually the same string, and for most backends there is no difference: one name, one
    /// program, one database. It matters where several backends drive one manager. `pacman` and
    /// `yay` in one config is an ordinary Arch machine — the repos from one, the AUR from the
    /// other — and both write `/var/lib/pacman/`. Keyed by their own names they were two locks
    /// over one database, so a sync touching both ran them concurrently and let pacman's own
    /// `db.lck` decide, which it does by failing the loser. The same holds for `apt`/`apt-get`
    /// and for `dnf`/`yum`/`microdnf`.
    ///
    /// The family table lives in `app::stale_lock`, because *which backends share a manager
    /// lock* and *which lock is left behind when one is killed* are the same fact, and the
    /// second copy of it would be the one that went stale.
    pub fn lock_key(&self) -> &str {
        crate::app::stale_lock::lock_key(&self.name)
    }

    /// The program that removes. Falls back to [`binary`](Self::binary), not to the name, so a
    /// user-defined noun with a separate remover still removes with the right tool.
    pub fn remove_binary(&self) -> &str {
        self.config
            .remove_binary
            .as_deref()
            .unwrap_or_else(|| self.binary())
    }

    /// The program that adds and removes repositories.
    pub fn repo_binary(&self) -> &str {
        self.config
            .repo_binary
            .as_deref()
            .unwrap_or_else(|| self.binary())
    }

    /// The program that removes a repository. Falls back to the one that adds it.
    pub fn repo_remove_binary(&self) -> &str {
        self.config
            .repo_remove_binary
            .as_deref()
            .unwrap_or_else(|| self.repo_binary())
    }

    /// The program that lists repositories.
    pub fn repo_list_binary(&self) -> &str {
        self.config
            .repo_list_binary
            .as_deref()
            .unwrap_or_else(|| self.binary())
    }
}

#[async_trait]
impl BackendCore for GenericBackendCore {
    fn name(&self) -> &str {
        &self.name
    }

    /// Every probe must be present, not just the first.
    ///
    /// A manager reached as a *plugin* of another program needs both halves: `kubectl krew …`
    /// works only because krew installed `kubectl-krew` on PATH, and a host with kubectl and no
    /// krew reported this backend READY and then failed every command with `unknown command
    /// "krew"` — including `shall update`, which refreshes every backend at once. That was
    /// found and fixed once, in a hand-written backend; expressing it here is what lets the
    /// hand-written one be deleted instead of kept for the one thing it knew.
    fn is_available(&self) -> bool {
        self.probes()
            .iter()
            .all(|p| self.executor.command_exists_sync(p))
    }

    fn probes(&self) -> Vec<String> {
        match &self.config.extra_probes {
            Some(extra) => std::iter::once(self.binary().to_string())
                .chain(extra.iter().cloned())
                .collect(),
            None => vec![self.binary().to_string()],
        }
    }

    fn needs_root(&self) -> bool {
        self.config.needs_root
    }

    // No `check_health` here. This was the better of two implementations of one sentence, and
    // the way to have one implementation is to have one — so it moved to
    // `core::manager::missing_program`, which every backend now shares, and this override was
    // deleted rather than reconciled.
}

#[async_trait]
impl MetadataProvider for GenericBackendCore {
    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let Some(probe) = &self.config.depends else {
            return Ok(vec![]);
        };

        // The operand is the argument that IS `{name}`, never one that merely contains it:
        // dnf asks with `--queryformat %{name}`, where those six characters are rpm's own
        // format language and substituting the package into them produces `%jq`.
        let has_operand = probe.args.iter().any(|a| a.as_str() == "{name}");
        // **A probe row without `{name}` is a broken row, not a command.** It used to run
        // verbatim — `dpkg -s` with no package asked the state of nothing — and its stdout was
        // parsed as the dependencies of whatever package was being planned, which is how one
        // bad row would have lied about every package at once.
        if !has_operand {
            return Err(Error::Validation(format!(
                "backend `{}`: its depends-probe row carries no `{{name}}`, so there is no \
                 way to ask about a specific package; fix the row",
                self.name
            )));
        }
        let mut final_args: Vec<String> = probe
            .args
            .iter()
            .filter(|a| a.as_str() != "{name}")
            .cloned()
            .collect();
        let bin = probe.binary.as_deref().unwrap_or(self.binary());
        crate::core::argv::push_names(&mut final_args, bin, [name]);

        let arg_refs: Vec<&str> = final_args.iter().map(|s| s.as_str()).collect();
        // Dependency resolution is a read-only query — never escalate with sudo.
        let output = self.executor.run_output(bin, &arg_refs, false).await?;

        Ok((probe.parse)(&output))
    }
}

/// Pull the dependency names out of a `depends` / `info --requires` report.
///
/// **Only what a dependency label introduces counts.** Most of what these commands print is not
/// a dependency: `zypper info --requires` opens with `Loading repository data...`, reports
/// `Installed      : No`, and prints a paragraph of prose — and taking the first word of every
/// non-empty line, which is what this did until 2026-07-30, yields packages called `Loading`,
/// `Reading` and `No`. The planner adds every dependency as an install node and then asks *that*
/// node for its dependencies, which returns the same three words, so the first real `zypper` run
/// in this project's history could not install anything at all: it died on a `requires` cycle
/// between three adverbs.
///
/// Two shapes are handled, and they are the two that exist:
///
/// - one dependency per labelled line — `  Depends: libc6` (apt)
/// - a labelled header and an indented block — `Requires : [4]` then the names (zypper)
///
/// A `Key : Value` row that is not a dependency label closes the block, which is what keeps
/// `Description`'s indented prose out.
pub fn parse_dependency_output(output: &str) -> Vec<String> {
    fn is_dependency_label(key: &str) -> bool {
        let k = key.trim().to_ascii_lowercase();
        k.starts_with("depends") || k.starts_with("requires") || k.starts_with("pre-depends")
    }
    // A count (`[4]`), a placeholder (`(none)`) or an empty value is a header, not a name.
    fn name_of(value: &str) -> Option<String> {
        let first = value.split_whitespace().next()?;
        if first.starts_with('[') || first.starts_with('(') || first == "<none>" {
            return None;
        }
        Some(first.to_string())
    }

    let mut deps = Vec::new();
    let mut in_block = false;
    for raw in output.lines() {
        if raw.trim().is_empty() {
            in_block = false;
            continue;
        }
        let indented = raw.starts_with([' ', '\t']);
        let trimmed = raw.trim();

        match trimmed.split_once(':') {
            Some((key, value)) if is_dependency_label(key) => {
                in_block = true;
                if let Some(n) = name_of(value) {
                    deps.push(n);
                }
            }
            // Any other top-level `Key : Value` row is metadata and ends the block. Indented
            // rows are left to fall through, because a dependency may legitimately carry a
            // colon (`libc.so.6(GLIBC_2.38)(64bit)` does not, but a path-shaped one would).
            Some(_) if !indented => in_block = false,
            _ => {
                if in_block && indented {
                    if let Some(n) = name_of(trimmed) {
                        deps.push(n);
                    }
                }
            }
        }
    }
    deps
}

//! One manager's settings, and the `{setting.KEY}` substitution that fills its argv.

use super::*;

/// Configuration for the Generic Manager Strategy.
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    pub name: String,
    /// The program to run, when it is not the backend's own name (XIII.12). `None` — every
    /// built-in — means the name is the command.
    pub binary: Option<String>,
    pub install_args: Vec<String>,
    pub remove_args: Vec<String>,
    /// Optional: the program that runs the REMOVE commands, when a manager uninstalls with a
    /// *separate* binary from the one it installs with — OpenBSD installs with `pkg_add` and
    /// removes with `pkg_delete`. `None` = removal uses the same binary as everything else.
    pub remove_binary: Option<String>,
    /// Args that also destroy the package's configuration (Debian's `purge`). `None` means
    /// this manager draws no such distinction, and `--purge` on it is refused rather than
    /// quietly doing an ordinary removal.
    pub purge_args: Option<Vec<String>>,
    pub list_args: Vec<String>,
    pub manual: ManualListing,
    /// Optional: args (run with `list_binary`) that report the packages the OS treats as
    /// essential, for the removal guard. `None` = the manager has no such concept.
    pub essential_args: Option<Vec<String>>,
    pub search_args: Vec<String>,
    /// Optional: if specified, use this binary for search instead of the backend name.
    pub search_binary: Option<String>,
    /// Optional: args that print every installable package name, one per line, and nothing
    /// else — what II.15's `re:` expands against. `None` means this manager cannot list its
    /// catalogue, which is the honest answer for every language registry, and a `re:` line
    /// naming it is refused rather than expanded to nothing.
    pub enumerate_args: Option<Vec<String>>,
    /// Optional: binary for `enumerate_args`, when the catalogue lives in a separate program
    /// (apt's is `apt-cache`, not `apt`).
    pub enumerate_binary: Option<String>,
    /// Optional: binary to run the LIST commands (`list_args`/`essential_args`) with,
    /// instead of the backend name. Required when a manager's query tool is a *separate*
    /// program — e.g. apt lists installed packages via `dpkg-query`, not `apt dpkg-query`.
    pub list_binary: Option<String>,
    pub upgrade_args: Vec<String>,
    pub update_args: Option<Vec<String>>,
    /// How to ask the manager what its own orphan verb *would* remove, without removing it.
    /// `None` means this manager cannot say, and a manager that cannot say does not remove.
    pub orphan_dry_run: Option<OrphanDryRun>,
    /// How to ask which installed packages this manager's repositories did not supply
    /// (`Queryable::foreign_to_repositories`). One bare name per line; `None` from a manager
    /// that draws no such distinction, which is all of them but pacman.
    pub foreign_args: Option<Vec<String>>,
    pub repo_add_args: Option<Vec<String>>,
    pub repo_remove_args: Option<Vec<String>>,
    pub repo_list_args: Option<Vec<String>>,
    /// Optional: the program that runs `repo_add_args`/`repo_remove_args`, when a manager
    /// edits its sources with a *separate* tool. apt's is `add-apt-repository` and apk's is a
    /// line appended to a file by `sh` — neither is `apt` or `apk`, and running them as
    /// subcommands of the manager is the same defect `list_binary` exists to prevent.
    pub repo_binary: Option<String>,
    /// Optional: the program that runs `repo_list_args`. Separate from `repo_binary` because a
    /// manager can write its sources one way and read them another (apk writes with `sh` and
    /// reads with `cat`). Falls back to `binary`, not to `repo_binary`.
    pub repo_list_binary: Option<String>,
    /// Optional: the program that runs `repo_remove_args`, when a manager adds a source one way
    /// and drops it another. Falls back to [`repo_binary`](Self::repo_binary).
    ///
    /// dnf adds with its own `config-manager` plugin and has no verb that removes: the drop-in
    /// file is deleted with `rm`. Two directions of the same operation, two programs — which is
    /// the argument `repo_list_binary` already carries for reading.
    pub repo_remove_binary: Option<String>,
    /// The shape of what `repo_list_args` prints.
    pub repo_list_shape: RepoListing,
    /// How to ask this manager what one package needs. `None` means it is not asked, which is
    /// the right answer for every manager whose own installer resolves its closure.
    pub depends: Option<DependsProbe>,
    /// How to empty this manager's download cache. `None` means it has no such verb.
    pub clean_cache: Option<CacheClean>,
    /// Native syntax for pinning an exact version at install (None = no version pinning).
    pub version_pin: Option<VersionPin>,
    /// Optional: the option key holding what `install_args` takes, when that is not the
    /// package's own name. `helm plugin install` takes a URL while `plugin list` and `plugin
    /// uninstall` speak the name from the plugin's `plugin.yaml` — so the name has to stay the
    /// identity (a declaration that names the URL installs once and can never be removed or
    /// recognised again), and the URL rides in an option. `None` = the name is the argument.
    pub install_source_option: Option<String>,
    pub needs_root: bool,
    pub is_exclusive: bool,
    /// Properties `info` learns by asking the manager a second question. Empty for a manager
    /// that reports none, which is most of them.
    pub property_probes: Vec<PropertyProbe>,
    /// A machine-readable listing to try before [`Self::list_args`], where the manager may
    /// have one (`Q43`). `None` means the text listing is all there is.
    pub machine_list: Option<MachineListing>,
    /// The manager's own "what has an update" command, where it has one (`Q44`). `None` means
    /// callers must ask about each package separately.
    pub outdated: Option<OutdatedProbe>,
    /// Where `search` gets its answers. Defaults to running `search_args`.
    pub search_source: SearchSource,
    /// This manager's names carry a qualifier the user need not type — see
    /// [`Searchable::qualifies_names`](crate::core::Searchable::qualifies_names).
    pub qualified_names: bool,
    /// This manager has no upgrade-all verb: upgrading means re-installing each installed
    /// package unpinned, with THESE args rather than `install_args`.
    ///
    /// The args are separate because they are not always the same: pub re-activates with the
    /// verb it installs with, while `cargo install foo` on an already-installed foo declines
    /// and needs `install --force`. A boolean would have upgraded cargo by asking it to do
    /// nothing, and reported success. `upgrade_args` is unused when this is `Some`.
    pub upgrade_reinstall_args: Option<Vec<String>>,
    /// Programs that must ALSO be present for this backend to be usable, beyond `binary`.
    /// `None` = the binary is the whole requirement, which is true of every manager that is a
    /// program rather than a plugin of one. See [`BackendCore::is_available`].
    pub extra_probes: Option<Vec<String>>,
    // `flag_map` was here: declared once, assigned at twenty-five registration sites, and read
    // at none. It was also absent from `CustomBackendDef`, so a user could not have set it even
    // if something had read it — a field that could never carry a fact into the program.
}

impl ManagerConfig {
    /// Substitute `{setting.KEY}` and `{setting.KEY|DEFAULT}` in every argv template from this
    /// backend's `[backend_settings]` block.
    ///
    /// **The one thing the data path could not say.** `conda` is environment-scoped: every verb
    /// carries `-n <env>`, where the env is a user choice read from
    /// `backend_settings.conda.env`. A `ManagerConfig` row is fixed at registration, so an argv
    /// that depends on the machine's settings had no way to be written as data — and the answer,
    /// for 319 lines, was a hand-written backend. `backend_is_data_not_code_tests.rs` records six
    /// exemptions blocked on the same shape.
    ///
    /// **Resolved once, at registration, not per call.** `backend_settings` is read from
    /// preferences before any backend registers and does not change during a run, so a per-call
    /// substitution would be the same answer computed N times — and the hand-written backends
    /// this replaces already resolved at registration (`conda.rs`'s `resolve_env(cfg)`).
    ///
    /// A placeholder that names a key with no value and no `|DEFAULT` is a **refusal**, not an
    /// empty string: `conda list -n --json` would ask conda about a flag rather than an
    /// environment, and a manager handed a malformed argv is exactly the silent-wrong-answer
    /// shape this repo keeps finding.
    pub fn resolve_settings(
        &mut self,
        settings: Option<&std::collections::HashMap<String, String>>,
    ) -> crate::core::Result<()> {
        let backend = self.name.clone();
        let lookup = |token: &str| -> Option<String> {
            let (key, fallback) = match token.split_once('|') {
                Some((k, d)) => (k, Some(d)),
                None => (token, None),
            };
            settings
                .and_then(|s| s.get(key))
                .filter(|v| !v.trim().is_empty())
                .cloned()
                .or_else(|| fallback.map(str::to_string))
        };

        let mut missing: Vec<String> = Vec::new();
        walk_args(self, &mut |arg: &mut String| {
            *arg = substitute(arg, &lookup, &mut missing);
        });

        if !missing.is_empty() {
            return Err(crate::core::Error::Config(format!(
                "`{backend}` needs `[backend_settings.{backend}]` to set {} — its argv is written \
                 against {} and there is no default. Add the key, or the commands this backend \
                 runs would be missing an operand.",
                missing.join(", "),
                missing
                    .iter()
                    .map(|k| format!("`{{setting.{k}}}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        // **The independent check, and the reason a forgotten field cannot ship.** `walk_args`
        // is a hand-written list of every argv-bearing field, and the failure mode of a hand-
        // written list is the field somebody adds next year — a placeholder in it would reach
        // the manager *literally*, as `-n {setting.env}`. This reads the whole struct through
        // `Debug`, which cannot miss a field because it did not write the list.
        let rendered = format!("{self:?}");
        if let Some(at) = rendered.find("{setting.") {
            return Err(crate::core::Error::Config(format!(
                "`{backend}`: a `{{setting.…}}` placeholder survived resolution — \
                 `generic::walk_args` does not visit the field holding it, so it would be passed \
                 to the manager verbatim. Near: {}",
                rendered[at..].chars().take(80).collect::<String>()
            )));
        }
        Ok(())
    }
}

/// Replace every `{setting.KEY}` / `{setting.KEY|DEFAULT}` in `arg`, recording keys that resolve
/// to nothing. Substitution is *inside* the token, so `--{setting.scope|system}` is one argument.
fn substitute(
    arg: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
    missing: &mut Vec<String>,
) -> String {
    const OPEN: &str = "{setting.";
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        let Some(end) = after.find('}') else {
            break; // an unclosed placeholder is not one; the Debug scan will catch it
        };
        out.push_str(&rest[..start]);
        let token = &after[..end];
        match lookup(token) {
            Some(value) => out.push_str(&value),
            None => {
                missing.push(token.split('|').next().unwrap_or(token).to_string());
                // Left in place so the Debug scan is not the thing that reports it; the
                // `missing` list names the key, which is what the user has to act on.
                out.push_str(&rest[start..start + OPEN.len() + end + 1]);
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Visit every argv token in a [`ManagerConfig`].
///
/// Hand-written, and checked by the `Debug` scan in [`ManagerConfig::resolve_settings`] rather
/// than trusted — a list like this is exactly the kind of thing that goes stale, and the whole
/// point of the pair is that going stale is a build failure rather than a wrong command.
fn walk_args(config: &mut ManagerConfig, visit: &mut dyn FnMut(&mut String)) {
    fn each(args: &mut [String], visit: &mut dyn FnMut(&mut String)) {
        for a in args {
            visit(a);
        }
    }
    fn maybe(args: &mut Option<Vec<String>>, visit: &mut dyn FnMut(&mut String)) {
        if let Some(a) = args {
            each(a, visit);
        }
    }

    each(&mut config.install_args, visit);
    each(&mut config.remove_args, visit);
    each(&mut config.list_args, visit);
    each(&mut config.search_args, visit);
    each(&mut config.upgrade_args, visit);
    maybe(&mut config.purge_args, visit);
    maybe(&mut config.essential_args, visit);
    maybe(&mut config.enumerate_args, visit);
    maybe(&mut config.update_args, visit);
    maybe(&mut config.repo_add_args, visit);
    maybe(&mut config.repo_remove_args, visit);
    maybe(&mut config.repo_list_args, visit);
    maybe(&mut config.upgrade_reinstall_args, visit);
    maybe(&mut config.extra_probes, visit);

    match &mut config.manual {
        ManualListing::Command { args, .. } | ManualListing::ExportFile { args, .. } => {
            each(args, visit)
        }
        ManualListing::AllInstalled | ManualListing::Unsupported => {}
    }
    if let Some(m) = &mut config.machine_list {
        each(&mut m.args, visit);
    }
    if let Some(o) = &mut config.outdated {
        each(&mut o.args, visit);
    }
    if let Some(o) = &mut config.orphan_dry_run {
        each(&mut o.args, visit);
    }
    if let Some(c) = &mut config.clean_cache {
        each(&mut c.args, visit);
    }
    if let Some(d) = &mut config.depends {
        each(&mut d.args, visit);
    }
    for p in &mut config.property_probes {
        each(&mut p.args, visit);
        visit(&mut p.template);
    }
    match &mut config.version_pin {
        Some(VersionPin::Inline(s)) => visit(s),
        Some(VersionPin::Before(args)) => each(args, visit),
        Some(VersionPin::After { args, unpinned }) => {
            each(args, visit);
            if let Some(u) = unpinned {
                visit(u);
            }
        }
        None => {}
    }
}

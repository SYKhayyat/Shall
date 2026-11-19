//! Install, remove, purge and the batching that keeps one argv under the platform's ceiling.

use super::*;

pub struct GenericInstallable {
    pub core: Arc<GenericBackendCore>,
}

/// The argument `install_args` takes for a backend whose install speaks a different vocabulary
/// than its list and remove (`install_source_option`).
///
/// Refusing beats guessing: deriving `diff` from `.../helm-diff` is right often enough to be
/// trusted and wrong often enough to install a plugin under a name nothing can remove, and the
/// name lives in the plugin's own `plugin.yaml`, which cannot be read before it is fetched.
fn install_source(backend: &str, spec: &PackageSpec, key: &str) -> Result<String> {
    spec.options
        .one(key)
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            crate::core::Error::Validation(format!(
            "{backend}:{name} needs `@{key}=…`. {backend} installs from that value but lists and \
             removes by name, so the declaration has to carry both: \
             `{backend}:{name}@{key}=<source>`.",
            backend = backend,
            name = spec.name,
            key = key,
        ))
        })
}

#[async_trait]
impl Installable for GenericInstallable {
    /// Answered from the same builder `install` runs through, so the exporter and the install
    /// cannot disagree about an argv. `Err` is `None`: a spec that cannot be turned into a
    /// command (an exact version this manager has no syntax for) is not something an artifact
    /// may quietly drop.
    fn planned_installs(&self, specs: &[PackageSpec]) -> Option<Vec<PlannedInstall>> {
        self.install_commands(specs).ok()
    }

    /// Answered from `remove_commands`, which is the builder `remove` itself runs through.
    ///
    /// `None` is the manager with no uninstall verb, and the exporter turns that into a named
    /// hole rather than an empty list (II.94) — an artifact that dropped a declared removal would
    /// describe a machine that still has the package.
    fn planned_removals(&self, specs: &[PackageSpec]) -> Option<Vec<PlannedInstall>> {
        self.remove_commands(specs)
    }

    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        if specs.is_empty() {
            return Ok(());
        }

        // A signature check this manager does is turned off by one line at a time, so specs
        // that disagree cannot share a command: the flag on a batch would hand one line's
        // opt-out to the next, which is the global switch `@unverified` is per-line to avoid.
        if capability::unverified_arg(&self.core.name).is_some() {
            let (opted, rest): (Vec<PackageSpec>, Vec<PackageSpec>) = specs
                .iter()
                .cloned()
                .partition(crate::core::download::is_unverified);
            if !opted.is_empty() && !rest.is_empty() {
                self.install_group(&opted, sudo).await?;
                return self.install_group(&rest, sudo).await;
            }
        }
        // `@system` splits a batch for the same reason and it is a sharper one: the flag says
        // *write into the environment the OS owns*, and handing one line's permission to the
        // next would install packages into a system python that nobody said that about (`Q49`).
        if capability::os_owned_env_arg(&self.core.name).is_some() {
            let (opted, rest): (Vec<PackageSpec>, Vec<PackageSpec>) = specs
                .iter()
                .cloned()
                .partition(crate::core::download::is_system);
            if !opted.is_empty() && !rest.is_empty() {
                self.install_group(&opted, sudo).await?;
                return self.install_group(&rest, sudo).await;
            }
        }
        // A leading-flag pin belongs to the subcommand, not to an operand, so it cannot be
        // shared: `cargo install --version 1.0 --version 2.0 -- a b` is not two pinned
        // installs. One command per spec, and only for the managers that spell it that way.
        if self
            .core
            .config
            .version_pin
            .as_ref()
            .is_some_and(VersionPin::is_one_per_command)
            && specs.len() > 1
        {
            for spec in specs {
                self.install_group(std::slice::from_ref(spec), sudo).await?;
            }
            return Ok(());
        }
        // **The operand caps, at the one door every batch walks through.** Windows'
        // CreateProcess takes 32 767 characters and unix ARG_MAX shrinks with the
        // environment; a 280-package adopted winget set or a thousand-name pip closure is
        // not a hypothetical. 100 names / 6 000 operand bytes per command sit far under both
        // and are what managers themselves batch to anyway. Recursing through `install`
        // (boxed: async recursion) re-runs every partition above for each chunk.
        if specs.len() > 1 {
            let take = batch_bound(specs);
            if take < specs.len() {
                self.install_group(&specs[..take], sudo).await?;
                return Box::pin(self.install(&specs[take..], sudo)).await;
            }
        }
        self.install_group(specs, sudo).await
    }

    async fn remove(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        // Some managers (e.g. Haskell's cabal/stack) genuinely have no uninstall verb.
        // An empty `remove_args` encodes that — UNLESS the manager removes with a separate
        // binary that is itself the verb (OpenBSD's `pkg_delete <name>`, no subcommand). A
        // separate remove binary means removal is supported however few args it takes.
        if self.core.config.remove_args.is_empty() && self.core.config.remove_binary.is_none() {
            return Err(crate::core::Error::Unsupported(self.core.name.clone()));
        }
        // **The batching is `remove_commands`'s, not a second copy here.** It splits at the same
        // operand caps the install side uses, so the executed removal and the exported one split
        // at identical points — a script that batches differently from the tool is a script that
        // fails on the machine where the tool would have worked.
        let specs: Vec<PackageSpec> = names
            .iter()
            .map(|n| PackageSpec {
                name: n.clone(),
                backend: self.core.name.clone(),
                ..Default::default()
            })
            .collect();
        let commands = self
            .remove_commands(&specs)
            .ok_or_else(|| crate::core::Error::Unsupported(self.core.name.clone()))?;
        // The guard is the `_reaped` this method was handed: it was spent in sync's preflight,
        // before any name reached here, and nothing below can widen what it authorised. Named at
        // the call site so the removal-surface gate still sees this path rather than reporting
        // `generic.rs` as a file that removes nothing.
        let _ = &_reaped;
        for command in commands {
            self.run_removal(command.args, &[], sudo).await?;
        }
        Ok(())
    }

    fn supports_purge(&self) -> bool {
        self.core.config.purge_args.is_some()
    }

    /// The pin syntax is the whole answer: a config that names one can build an install argument
    /// from a version, and a config that names none cannot (`Q53`).
    fn pins_version(&self) -> bool {
        self.core.config.version_pin.is_some()
    }

    async fn purge(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        let Some(args) = self.core.config.purge_args.clone() else {
            return Err(crate::core::Error::Unsupported(format!(
                "{} has no purge — it does not keep a package's configuration apart from the \
                 package",
                self.core.name
            )));
        };
        self.run_removal(args, names, sudo).await
    }
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// Operand caps for one generated command: names and their total bytes. Windows' CreateProcess
/// takes 32 767 characters and unix ARG_MAX shrinks with the environment; these sit far under
/// both, and are the sizes managers themselves batch to anyway.
pub const BATCH_MAX_NAMES: usize = 100;
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub const BATCH_MAX_OPERAND_BYTES: usize = 6_000;

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// How many leading specs fit one command's operand budget. At least one — a single name over
/// the byte cap is still attempted alone, because truncating it away would silently skip it
/// and one oversized name failing loudly is the honest outcome.
pub fn batch_bound(specs: &[PackageSpec]) -> usize {
    let mut bytes = 0usize;
    specs
        .iter()
        .take(BATCH_MAX_NAMES)
        .take_while(|s| {
            bytes += s.name.len() + 1;
            bytes <= BATCH_MAX_OPERAND_BYTES
        })
        .count()
        .max(1)
}

impl GenericInstallable {
    /// One install command for specs that agree about verification.
    async fn install_group(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        for planned in self.install_commands(specs)? {
            let arg_refs: Vec<&str> = planned.args.iter().map(|s| s.as_str()).collect();
            let outcome = if self.core.config.is_exclusive {
                self.core
                    .executor
                    .run_exclusive(self.core.lock_key(), &planned.binary, &arg_refs, sudo)
                    .await
            } else {
                self.core
                    .executor
                    .run(&planned.binary, &arg_refs, sudo)
                    .await
            };
            // The chain the flag would ride on, for the advice below: `plugin install` for helm.
            let chain: Vec<String> = planned
                .args
                .iter()
                .take_while(|a| !a.starts_with('-'))
                .cloned()
                .collect();
            outcome
                .map_err(|e| self.explain_verification(e, planned.unverified, &chain))
                .map_err(|e| self.explain_os_owned_environment(e))?;
        }
        Ok(())
    }

    /// The commands `install` would run for these specs, in order, without running any of them.
    ///
    /// **One builder, read or run.** `#67`'s export needs the argv that would reproduce a
    /// package on a fresh machine, and the only honest place to get it is the code that builds
    /// the argv a sync actually runs — a second builder for the exporter would be free to
    /// disagree with the first the next time a capability flag moved, which is `#151` and `#162`
    /// in another dress. So the tail of `install_group` became this, and the install became a
    /// loop over its answer.
    pub(crate) fn install_commands(&self, specs: &[PackageSpec]) -> Result<Vec<PlannedInstall>> {
        let mut out = Vec::new();
        self.collect_install_commands(specs, &mut out)?;
        Ok(out)
    }

    fn collect_install_commands(
        &self,
        specs: &[PackageSpec],
        out: &mut Vec<PlannedInstall>,
    ) -> Result<()> {
        if specs.is_empty() {
            return Ok(());
        }

        let mut final_args: Vec<String> = self.core.config.install_args.clone();
        let mut names: Vec<String> = Vec::with_capacity(specs.len());
        // A pin that puts an option *after* the name it pins (`gem install jq -v 1.6`) is the
        // one shape the terminator cannot precede — behind `--` that `-v` is a package.
        let mut trailing_option = false;
        let mut leading: Vec<String> = Vec::new();
        for spec in specs {
            if let Some(key) = &self.core.config.install_source_option {
                names.push(install_source(&self.core.name, spec, key)?);
                continue;
            }
            // Honor an exact version pin (reproducible/locked installs) using the
            // backend's native syntax, when both a pin syntax and a concrete version exist.
            match (spec.options.one("version"), &self.core.config.version_pin) {
                (Some(ver), Some(pin)) if is_concrete_version(ver) => {
                    // A leading arg goes into the subcommand args, ahead of the terminator,
                    // and the name stays behind it — which is the safer of the two shapes and
                    // is available whenever the flag is an option of the verb, not the operand.
                    leading.extend(pin.leading_args(ver));
                    trailing_option |= pin.emits_trailing_option();
                    names.extend(pin.apply(&spec.name, ver));
                }
                // A manager that will not install without a version gets the one it accepts
                // for "current". Without this, `asdf:nodejs` builds `asdf install nodejs` and
                // asdf rejects it — an argv Shall constructs perfectly and the tool refuses,
                // which is E13's family.
                (_, Some(pin)) if pin.unpinned().is_some() => {
                    let fallback = pin.unpinned().unwrap_or_default().to_string();
                    // Asked of the pin, not asserted: this branch used to set the flag
                    // unconditionally, so a *fallback* version dropped the terminator even
                    // when the fallback was an operand (`asdf install nodejs latest`). Both
                    // branches now answer the same question the same way.
                    trailing_option |= pin.emits_trailing_option();
                    names.extend(pin.apply(&spec.name, &fallback));
                }
                // A concrete version with no syntax to spend it on. The planner refuses this by
                // name before anything runs (`Q53`), so reaching here means something built a
                // spec without going through a plan — and dropping the pin silently is the one
                // outcome worse than either honouring it or refusing it, because the install
                // then reports success at a version nobody asked for.
                (Some(ver), None) if is_concrete_version(ver) => {
                    return Err(crate::core::Error::Unsupported(format!(
                        "`{}` cannot install an exact version, so `{}@version={}` cannot be met{}",
                        self.core.name,
                        spec.name,
                        ver,
                        match capability::cannot_pin_reason(&self.core.name) {
                            Some(why) => format!(" — {}", why),
                            None => String::new(),
                        }
                    )));
                }
                _ => names.push(spec.name.clone()),
            }
        }
        final_args.extend(leading);

        // A pin that emits a trailing option (`gem install jq -v 1.6`) cannot share a
        // command with a plain package: the option would constrain the wrong member.
        // Like the leading-flag case above, one command per spec when any trailing pin
        // is present.
        if trailing_option && specs.len() > 1 {
            for spec in specs {
                self.collect_install_commands(std::slice::from_ref(spec), out)?;
            }
            return Ok(());
        }

        // Before the terminator: behind `--` this is a package name.
        let opting_out = specs.iter().any(crate::core::download::is_unverified);
        if opting_out {
            if let Some(arg) = capability::unverified_arg(&self.core.name) {
                // Asked, not assumed (G-8). `--verify=false` is helm 4's flag; helm 3 rejects
                // it with `unknown flag: --verify`, so emitting it unconditionally traded
                // E11's argv defect for another one on every helm 3 in existence. The
                // subcommand chain is the non-flag prefix of what has been built so far —
                // `plugin install` for helm — because that is the help that documents it.
                let chain: Vec<String> = final_args
                    .iter()
                    .take_while(|a| !a.starts_with('-'))
                    .cloned()
                    .collect();
                // Withheld only on positive evidence that the tool rejects it. `None` — no
                // such program here, or its help would not run — leaves the capability table
                // in charge, because a probe that cannot ask has learned nothing.
                //
                // And withheld SILENTLY (Q14, ruled 2026-07-30; V.104). helm 3 does not verify
                // plugins at all, so the state `@unverified` asks for is the state the machine
                // is already in: "accepted and already true" is a correct no-op, not a defect,
                // and a warning on a run that did the right thing teaches people that warnings
                // are noise. The case where silence WOULD be wrong — a tool that verifies under
                // a flag it has since renamed — is not lost: the install then fails, and
                // `verification_note` says so at the one moment the distinction matters.
                if crate::core::tool_help::accepts_flag(self.core.binary(), &chain, arg)
                    != Some(false)
                {
                    final_args.push(arg.to_string());
                }
            }
        }

        // Also before the terminator, and asked of the tool for the same reason the flag above
        // is: `--break-system-packages` arrived in pip 23.0.1, and an older pip answers
        // `no such option`, which would turn a refusal a user can act on into an argv defect
        // they cannot (`Q49`).
        let into_os_env = specs.iter().any(crate::core::download::is_system);
        if into_os_env {
            if let Some(arg) = capability::os_owned_env_arg(&self.core.name) {
                let chain: Vec<String> = final_args
                    .iter()
                    .take_while(|a| !a.starts_with('-'))
                    .cloned()
                    .collect();
                if crate::core::tool_help::accepts_flag(self.core.binary(), &chain, arg)
                    != Some(false)
                {
                    final_args.push(arg.to_string());
                }
            }
        }

        if trailing_option {
            final_args.extend(names);
        } else {
            crate::core::argv::push_names(&mut final_args, self.core.binary(), names);
        }

        out.push(PlannedInstall {
            binary: self.core.binary().to_string(),
            args: final_args,
            unverified: opting_out,
        });
        Ok(())
    }

    /// PEP 668, in words that name the two things a user can actually do (`Q49`).
    ///
    /// pip's own refusal is a wall of text about `--break-system-packages`, virtual
    /// environments and `pipx`, addressed to somebody typing `pip install` — not to somebody
    /// who wrote a line in a manifest and does not know which of those Shall supports. Both
    /// answers it names here are ones a declaration can hold: `pipx:` is a backend Shall ships,
    /// and `@system=true` is the flag that flips this exact refusal.
    fn explain_os_owned_environment(&self, e: crate::core::Error) -> crate::core::Error {
        if capability::os_owned_env_arg(&self.core.name).is_none() {
            return e;
        }
        let crate::core::Error::CommandFailed {
            message,
            retry,
            absent_name,
        } = e
        else {
            return e;
        };
        // The marker pip prints, and nothing else: matching on "externally managed" prose alone
        // would also catch a package whose description contains it.
        if !message.contains("externally-managed-environment") {
            return crate::core::Error::CommandFailed {
                message,
                retry,
                absent_name,
            };
        }
        crate::core::Error::CommandFailed {
            message: format!(
                "{}\n  This Python belongs to the operating system's package manager, which is \
                 what PEP 668's marker file says, and `{}` will not write into it.\n  \
                 What a declaration can do about it:\n    \
                 pipx:{name}         install it in its own environment — the tool built for \
                 this, and a backend Shall already drives\n    \
                 {backend}:{name}@system=true   write into the system Python anyway, on this \
                 line only",
                message.trim_end(),
                self.core.binary(),
                backend = self.core.name,
                // The refusal is about a set; naming one of them is enough to show the shape,
                // and a list of forty would bury the two verbs that matter.
                name = "<package>",
            ),
            retry,
            absent_name,
        }
    }

    /// A manager that refuses an unsignable source names its own flag, which no declaration
    /// can write. Point at the one that can — and only when there is one.
    fn explain_verification(
        &self,
        e: crate::core::Error,
        opting_out: bool,
        chain: &[String],
    ) -> crate::core::Error {
        let Some(flag) = capability::unverified_arg(&self.core.name) else {
            return e;
        };
        let crate::core::Error::CommandFailed {
            message,
            retry,
            absent_name,
        } = e
        else {
            return e;
        };
        // Asked, not assumed — the same probe the emission path uses, so the advice and the
        // argv cannot disagree about whether the flag went out.
        let withheld =
            crate::core::tool_help::accepts_flag(self.core.binary(), chain, flag) == Some(false);
        let note = verification_note(
            &self.core.name,
            self.core.binary(),
            flag,
            opting_out,
            withheld,
            &message,
        );
        crate::core::Error::CommandFailed {
            message: match note {
                Some(note) => format!("{}\n  {}", message.trim_end(), note),
                None => message,
            },
            retry,
            absent_name,
        }
    }

    /// Execute a removal-shaped command. Both `remove` and `purge` come through here.
    ///
    /// Takes the subcommand args and the operands separately, because `purge` has its own args
    /// and no builder of its own — the only thing they share is "append the names, then run".
    /// `remove` passes args that already carry its names appended by `remove_commands`, and the
    /// empty list it passes for `names` is what makes this a plain run rather than an append.
    async fn run_removal(&self, mut args: Vec<String>, names: &[String], sudo: bool) -> Result<()> {
        if !names.is_empty() {
            let bin = self.core.remove_binary();
            crate::core::argv::push_names(&mut args, bin, names);
        }
        if args.is_empty() {
            return Ok(());
        }
        let bin = self.core.remove_binary();
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        if self.core.config.is_exclusive {
            self.core
                .executor
                .run_exclusive(self.core.lock_key(), bin, &arg_refs, sudo)
                .await?;
        } else {
            self.core.executor.run(bin, &arg_refs, sudo).await?;
        }
        Ok(())
    }

    /// The argv `remove` would run, without running it.
    ///
    /// **One builder, two callers.** `run_removal` executes it; `planned_removals` returns it to
    /// the exporter. A second spelling of "how this manager removes" would be free to disagree
    /// with the first the moment `remove_args` or the binary changes, which is the failure
    /// `planned_installs` was written to avoid on the install side.
    ///
    /// `None` when the manager has no removal at all — the same condition `remove` refuses on, so
    /// the exporter reports the hole rather than emitting a command that could never run.
    fn remove_commands(&self, specs: &[PackageSpec]) -> Option<Vec<PlannedInstall>> {
        if specs.is_empty() {
            return Some(Vec::new());
        }
        if self.core.config.remove_args.is_empty() && self.core.config.remove_binary.is_none() {
            return None;
        }
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        // The same operand caps `remove` uses, and for the same measured reason: a removal of 280
        // adopted packages builds the same oversized command an install would have. Batched here
        // rather than in the caller so the artifact and the tool split at identical points.
        let bin = self.core.remove_binary().to_string();
        let mut out = Vec::new();
        let mut rest: &[String] = &names;
        while !rest.is_empty() {
            let mut bytes = 0usize;
            let take = rest
                .iter()
                .take(BATCH_MAX_NAMES)
                .take_while(|n| {
                    bytes += n.len() + 1;
                    bytes <= BATCH_MAX_OPERAND_BYTES
                })
                .count()
                .max(1);
            let mut args = self.core.config.remove_args.clone();
            crate::core::argv::push_names(&mut args, &bin, &rest[..take]);
            out.push(PlannedInstall {
                binary: bin.clone(),
                args,
                unverified: false,
            });
            rest = &rest[take..];
        }
        Some(out)
    }
}

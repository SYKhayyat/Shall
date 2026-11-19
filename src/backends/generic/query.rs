//! Answering questions about the machine: installed listings, versions, properties, orphans.

use super::*;

pub struct GenericQueryable {
    pub core: Arc<GenericBackendCore>,
}

#[async_trait]
impl Queryable for GenericQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        // Ask for the machine-readable listing first, if this manager might have one (Q43).
        // A refusal is an answer — this tool is too old — and costs one invocation per run,
        // because the listing memo means a run gets here once per manager.
        if let Some(machine) = &self.core.config.machine_list {
            let bin = machine
                .binary
                .as_deref()
                .or(self.core.config.list_binary.as_deref())
                .unwrap_or(self.core.binary());
            let args: Vec<&str> = machine.args.iter().map(|s| s.as_str()).collect();
            match self.core.executor.probe_output(bin, &args).await {
                Ok(output) => return Ok((machine.parse)(&output)?),
                Err(e) => debug!(
                    "`{} {}` was refused, so `{}` is being read from its text listing instead \
                     — an older {} that does not have the flag: {e}",
                    bin,
                    machine.args.join(" "),
                    self.core.name,
                    self.core.name,
                ),
            }
        }
        let args: Vec<&str> = self
            .core
            .config
            .list_args
            .iter()
            .map(|s| s.as_str())
            .collect();
        // Use the configured list binary if the query tool is a separate program (e.g.
        // apt -> dpkg-query); otherwise the backend's own binary.
        let bin = self
            .core
            .config
            .list_binary
            .as_deref()
            .unwrap_or(self.core.binary());
        let output = self.core.executor.run_output(bin, &args, false).await?;
        Ok(self.core.parser.parse_installed(&output)?)
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        match &self.core.config.manual {
            ManualListing::AllInstalled => self.list_installed().await,
            ManualListing::Command {
                binary,
                args,
                format,
            } => {
                let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let bin = binary
                    .as_deref()
                    .or(self.core.config.list_binary.as_deref())
                    .unwrap_or(self.core.binary());
                let output = self.core.executor.run_output(bin, &args, false).await?;
                Ok(match format {
                    ManualFormat::BareNames => {
                        crate::parsers::parse_bare_names(&output, &self.core.name)?
                    }
                    ManualFormat::SameAsInstalled => self.core.parser.parse_installed(&output)?,
                    ManualFormat::Read(read) => read(&output)?,
                })
            }
            ManualListing::ExportFile {
                binary,
                args,
                format,
            } => {
                self.list_from_export(binary.as_deref(), args, *format)
                    .await
            }
            // Deliberately empty, not `list_installed`. Callers gate on `tracks_manual`;
            // returning the installed set here would be a confident wrong answer.
            ManualListing::Unsupported => Ok(Vec::new()),
        }
    }

    fn tracks_manual(&self) -> bool {
        !matches!(self.core.config.manual, ManualListing::Unsupported)
    }

    fn manual_source(&self) -> String {
        match &self.core.config.manual {
            ManualListing::AllInstalled => format!(
                "everything {} installed ({0} installs no dependencies of its own)",
                self.core.name
            ),
            ManualListing::Command { binary, args, .. } => {
                let bin = binary
                    .as_deref()
                    .or(self.core.config.list_binary.as_deref())
                    .unwrap_or(self.core.binary());
                format!("{} {}", bin, args.join(" "))
            }
            ManualListing::ExportFile { binary, args, .. } => {
                let bin = binary
                    .as_deref()
                    .or(self.core.config.list_binary.as_deref())
                    .unwrap_or(self.core.binary());
                format!(
                    "{} {} — what {} can reinstall, not everything it can see",
                    bin,
                    args.join(" "),
                    self.core.name
                )
            }
            ManualListing::Unsupported => {
                format!(
                    "{} cannot tell your choices from dependencies",
                    self.core.name
                )
            }
        }
    }

    /// Read through `probe_output`, so a manager that refuses the flag is *unknown* rather
    /// than *nothing is foreign*. An empty answer read out of a failure would attribute every
    /// AUR package to pacman again, which is the state this exists to leave.
    async fn foreign_to_repositories(&self) -> Result<Option<Vec<String>>> {
        let Some(ref foreign_args) = self.core.config.foreign_args else {
            return Ok(None);
        };
        let args: Vec<&str> = foreign_args.iter().map(|s| s.as_str()).collect();
        let bin = self
            .core
            .config
            .list_binary
            .as_deref()
            .unwrap_or(self.core.binary());
        let output = self.core.executor.probe_output(bin, &args).await?;
        Ok(Some(
            output
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(|l| l.split_whitespace().next().unwrap_or(l).to_string())
                .collect(),
        ))
    }

    async fn fetch_essential(&self) -> Result<Vec<String>> {
        let Some(ref essential_args) = self.core.config.essential_args else {
            return Ok(Vec::new());
        };
        let args: Vec<&str> = essential_args.iter().map(|s| s.as_str()).collect();
        let bin = self
            .core
            .config
            .list_binary
            .as_deref()
            .unwrap_or(self.core.binary());
        let output = self.core.executor.run_output(bin, &args, false).await?;
        Ok(self.core.parser.parse_essential(&output))
    }

    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let all = self.installed_listing().await?;
        // Windows package managers use CASE-INSENSITIVE ids, but their list output frequently
        // returns a different casing than the install id: choco installs "wget" yet lists the
        // Title "Wget", so a case-sensitive `p.name == name` misses it and the remove is
        // silently skipped (package + manifest left behind). Match case-insensitively for
        // those. winget additionally records a vendor-qualified Id ("jqlang.jq") that is
        // commonly installed/removed by its bare moniker ("jq"), so also accept the trailing
        // dot-segment. Kept scoped to Windows managers to avoid mis-matching legitimately
        // case-distinct or dotted names elsewhere (e.g. npm "socket.io").
        let b = self.core.name.as_str();
        let ci = matches!(b, "choco" | "scoop" | "winget");
        let winget = b == "winget";
        let found = all
            .iter()
            .find(|p| {
                p.name == name
                    || (ci && p.name.eq_ignore_ascii_case(name))
                    || (winget
                        && p.name
                            .rsplit('.')
                            .next()
                            .is_some_and(|s| s.eq_ignore_ascii_case(name)))
            })
            .cloned();
        let Some(mut pkg) = found else {
            return Ok(None);
        };
        // Concurrently: each probe is a subprocess and they have nothing to say to one
        // another. The outer fan-out overlaps *packages*, so a serial loop here is P extra
        // serial spawns inside each of them rather than a serial run overall — which is
        // exactly the shape that hides. The width is the number of probes configured for this
        // backend, not a cap: it is a statement about the list, and a user who declared four
        // property probes asked for four questions, not for them to be rationed.
        let probes = &self.core.config.property_probes;
        let answers = futures::future::join_all(probes.iter().map(|probe| async move {
            probe
                .resolve(&self.core, name)
                .await
                .map(|value| (probe.property.clone(), value))
        }))
        .await;
        pkg.properties.extend(answers.into_iter().flatten());
        Ok(Some(pkg))
    }
}

impl GenericQueryable {
    /// Run a manager's own export and read the set back out of the file it wrote.
    ///
    /// The temp directory must outlive the read, so it is bound to a name rather than
    /// left temporary in an expression — dropping it deletes the file the manager just
    /// wrote, and the read that follows would find nothing.
    ///
    /// **A missing file is an error, never an empty set.** An export that did not run is
    /// indistinguishable from a machine with nothing on it if this returns `Ok(vec![])`, and
    /// the caller is `adopt` — which would take that as "nothing to adopt" and say so.
    async fn list_from_export(
        &self,
        binary: Option<&str>,
        args: &[String],
        format: ExportFormat,
    ) -> Result<Vec<Package>> {
        let bin = binary
            .or(self.core.config.list_binary.as_deref())
            .unwrap_or(self.core.binary());
        let dir = tempfile::Builder::new()
            .prefix("shall-export-")
            .tempdir()
            .map_err(|e| {
                Error::Io(format!(
                    "could not make a directory for `{bin}`'s export: {e}"
                ))
            })?;
        let path = dir.path().join("export.json");
        let path_arg = path.to_string_lossy().to_string();
        let rendered: Vec<String> = args
            .iter()
            .map(|a| a.replace("{file}", &path_arg))
            .collect();
        let argv: Vec<&str> = rendered.iter().map(|s| s.as_str()).collect();
        // The manager's stdout names what it declined to export, one line per package, in the
        // user's own display language. It is not the set and is not parsed for one; the count
        // below is a set difference, which no localisation changes.
        let _ = self.core.executor.run_output(bin, &argv, false).await?;
        let text = tokio::fs::read_to_string(&path).await.map_err(|e| {
            Error::Io(format!(
                "`{bin} {}` wrote no export for Shall to read ({e}). Nothing was adopted from \
                 `{}` — rather than reporting the machine as empty.",
                rendered.join(" "),
                self.core.name
            ))
        })?;
        let restorable = match format {
            ExportFormat::WingetJson => crate::parsers::windows::parse_winget_export(&text)?,
        };
        self.report_unrestorable(&restorable).await;
        Ok(restorable)
    }

    /// Say how much of the machine this manager cannot put back, and name a few.
    ///
    /// Adoption that silently drops two thirds of what `list` shows reads as a bug in
    /// adoption. It is not: those rows name things the manager can uninstall and can never
    /// install. Counted as a set difference against the listing rather than by matching the
    /// manager's own message, which is localised and would silently count zero abroad.
    async fn report_unrestorable(&self, restorable: &[Package]) {
        let Ok(installed) = self.list_installed().await else {
            return;
        };
        let keep: std::collections::HashSet<&str> =
            restorable.iter().map(|p| p.name.as_str()).collect();
        let dropped: Vec<&str> = installed
            .iter()
            .map(|p| p.name.as_str())
            .filter(|n| !keep.contains(n))
            .collect();
        if dropped.is_empty() {
            return;
        }
        let sample: Vec<&str> = dropped.iter().take(3).copied().collect();
        // `warn`, not `info`: a default run prints `warn` and above, and "two thirds of your
        // installed software is outside management" is a fact about the outcome the user just
        // asked for. At `info` it would be invisible to everyone who did not already suspect it.
        warn!(
            "{name} lists {total} installed entries and can reinstall {kept} of them, adopted \
             here as {distinct} declaration(s). The other {gone} — e.g. {sample} — are entries \
             {name} can remove but never install, so a declaration naming one could never \
             converge, and they were left out.",
            name = self.core.name,
            total = installed.len(),
            kept = installed.len() - dropped.len(),
            distinct = restorable.len(),
            gone = dropped.len(),
            sample = sample.join(", "),
        );
    }
}

impl std::fmt::Debug for OutdatedProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutdatedProbe")
            .field("binary", &self.binary)
            .field("args", &self.args)
            .finish_non_exhaustive()
    }
}

/// A property of an installed package that only the manager can answer, and the command that
/// asks it.
///
/// `npm prefix -g`, `pipx environment --value PIPX_HOME`, `yarn global bin`: a second command
/// whose stdout is a directory, and a template that turns it into this package's value. Six
/// hand-written backends existed largely for this — it is what `info` needed and the generic
/// queryable could not do.
///
/// A **list**, not one install path, because `shall info` prints every property a package
/// carries: npm, pnpm and yarn each report `bin_path` beside `install_path`, and collapsing
/// that to one probe would have quietly removed a line from a user's output.
#[derive(Debug, Clone)]
pub struct PropertyProbe {
    /// The property key, as `shall info` prints it — `install_path`, `bin_path`.
    pub property: String,
    /// Argv run against the backend's binary; its stdout is the base value.
    pub args: Vec<String>,
    /// `{base}` and `{name}` substituted. `{base}` alone is a legitimate template: `bin_path`
    /// is the directory itself, with no per-package component.
    pub template: String,
}

impl PropertyProbe {
    /// `None` rather than an error on every failure path: these are enrichment for `shall
    /// info`, and a manager that will not answer must not turn a working `info` into a failed
    /// one — which is what the hand-written backends did, each in its own words.
    async fn resolve(&self, core: &GenericBackendCore, name: &str) -> Option<String> {
        let args: Vec<&str> = self.args.iter().map(String::as_str).collect();
        // **Once per run, and the argv is why that is sound.** `{name}` is substituted into the
        // template below and never into these args, so the answer depends on `cmd` + `args` and on
        // nothing else — and the outer fan-outs ask about many packages at once, so without the
        // memo `shall info` over twenty packages ran `npm prefix -g` twenty times to learn one
        // prefix (`PLAN.md` #80).
        let base = core
            .executor
            .installed_listings()
            .run_output_once_per_run(&core.executor, core.binary(), &args, false)
            .await
            .ok()?;
        let base = base.trim();
        if base.is_empty() {
            return None;
        }
        Some(
            self.template
                .replace("{base}", base)
                .replace("{name}", name),
        )
    }
}

/// Where a manager's `search` answers come from.
///
/// Not every manager has one. npm's is slow and output-unstable, pnpm has none, and yarn
/// removed its own in Berry — all three resolve from the same npm registry, which is why
/// `node_registry.rs` exists and why three separate backends reached for it. A search that is
/// an HTTP call rather than a subcommand is still this backend's search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSource {
    /// Run `search_args` and parse stdout. Every manager that has a real search subcommand.
    Command,
    /// Query the public npm registry over HTTP, tagging results with this backend's name.
    NpmRegistry,
    /// Look the exact name up on PyPI over HTTP.
    ///
    /// pip's own `search` was disabled upstream — PyPI withdrew the XML-RPC endpoint over
    /// abuse — and there is no public full-text replacement, so this is name resolution rather
    /// than discovery. That is a smaller answer than a real search and still this backend's
    /// search; answering "not configured" instead would refuse a question pip can answer.
    PyPi,
}

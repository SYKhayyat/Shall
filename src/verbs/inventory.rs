//! **The read-only verbs that describe what is here: `audit`, `sbom`, `export`, `why`.**
//!
//! None of them changes anything, and each answers a different question about the same machine —
//! what is vulnerable, what is installed, what would move to another machine, and what put this
//! package here. They were spread through a file called `history`.

use crate::verbs::prelude::*;

pub async fn handle_audit(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    state: &tokio::sync::Mutex<crate::core::StateRegistry>,
    out: Output,
) -> Result<()> {
    let report = crate::app::insight::audit(config, registry, state).await?;
    crate::app::insight::print_audit(&report, out).map_err(|e| e.into())
}

pub async fn handle_sbom(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    state: &tokio::sync::Mutex<crate::core::StateRegistry>,
) -> Result<()> {
    println!(
        "{}",
        crate::app::insight::sbom(config, registry, state).await?
    );
    Ok(())
}

pub async fn handle_export(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    state: &tokio::sync::Mutex<crate::core::StateRegistry>,
    format: Option<&str>,
    out: &str,
    stdout: bool,
    force: bool,
) -> Result<()> {
    use crate::app::export::{export, Format, Outcome};
    let fmt = match format {
        Some(s) => Some(
            Format::parse(s)
                .with_context(|| format!("unknown export format '{}' (brew|pip|npm|apt)", s))?,
        ),
        None => None,
    };
    if stdout && fmt.is_none() {
        anyhow::bail!("--stdout needs a single --format (brew|pip|npm|apt).");
    }
    let out_dir = std::path::PathBuf::from(out);
    let results = export(state, registry, config, fmt, &out_dir, stdout, force).await?;
    for (file, outcome) in &results {
        match outcome {
            Outcome::NoPackages => println!("  skipped {} (no matching packages)", file),
            Outcome::Wrote(path) => println!("  wrote   {}", path.display()),
            Outcome::WouldWrite(path) => {
                println!(
                    "  {} would write {}",
                    crate::core::dry_run::MARKER,
                    path.display()
                )
            }
            Outcome::WroteBeside { taken, renamed } => {
                println!("  wrote   {}", renamed.display());
                println!(
                    "          ({} already exists and was left alone; re-run with --force to replace it)",
                    taken.display()
                );
            }
        }
    }
    Ok(())
}

/// `shall export --from-declaration --format bash` — a script that reproduces the machine
/// without Shall (#67, II.94).
///
/// **The commands come from the backends' own argv builder**, `planned_installs`, which is the
/// same code `install` runs through. Nothing here knows how `apt` installs anything, and nothing
/// here runs anything: an export never touches the machine.
///
/// **Every declared package is exported, not only the ones missing here.** The artifact is for a
/// fresh machine, where none of them are installed; a script built from this machine's
/// differences would print nothing on the one machine that already has everything and install
/// nothing on the one that has none.
pub async fn handle_script_export(
    app: &App,
    source: crate::app::bootstrap::Source,
    format: crate::app::bootstrap::Format,
    best_effort: bool,
    out: &str,
    force: bool,
) -> Result<()> {
    // Both sources land on the same loop: a map backend -> specs, which the argv builder turns
    // into commands. The two sources differ only in what goes in the map and what must be
    // omitted (a scan sees no secrets and no opted-out lines — it is what the machine *is*).
    let mut script = crate::app::bootstrap::Script::default();
    let mut by_backend: std::collections::BTreeMap<String, Vec<crate::core::PackageSpec>> =
        std::collections::BTreeMap::new();
    let priority;
    let mut omitted: Vec<crate::app::bootstrap::Omission> = Vec::new();

    match source {
        crate::app::bootstrap::Source::Declarations => {
            let resolver = crate::app::sync::resolver::StateResolver::new(
                &app.config,
                app.registry.clone(),
                false,
            )
            .await;
            let state = resolver.resolve_model().await?;
            priority = resolver.priority_for_host().await?;
            omitted.extend(unrepresentable(&state));
            for (backend, specs) in &state.packages {
                // An opted-out line stays declared and stays synced; it is simply not something
                // the artifact has to be able to say (II.94), so it is not a hole either.
                let specs: Vec<crate::core::PackageSpec> = specs
                    .iter()
                    .filter(|s| s.present && s.options.one("export") != Some("no"))
                    .cloned()
                    .collect();
                if !specs.is_empty() {
                    by_backend.insert(backend.clone(), specs);
                }
            }
        }
        crate::app::bootstrap::Source::Scan => {
            let discovery = app.adopter().await.discover().await?;
            priority = crate::model::priority::Priority::default();
            for pkg in &discovery.adopt {
                let key = pkg.backend.clone();
                by_backend.entry(key).or_default().push(construct_spec(pkg));
            }
            // The artifact is what adopt would take, which is the honest scan: a machine's
            // next bootstrap is the set of things Shall would claim on a fresh copy of it.
            if by_backend.is_empty() {
                return Err(crate::core::Error::Differences(
                    "the scan found nothing `shall adopt` would claim. Nothing was written.".into(),
                )
                .into());
            }
        }
    };
    emit_commands(app, &mut script, by_backend, priority, &mut omitted);
    script.omitted = omitted;

    // II.94: an export that cannot say everything prints nothing and names the lines — unless
    // somebody asked for the partial artifact on purpose.
    if !script.omitted.is_empty() && !best_effort {
        let named: Vec<String> = script
            .omitted
            .iter()
            .map(|o| format!("  {} — {}", o.line, o.why))
            .collect();
        return Err(crate::core::Error::Differences(format!(
            "this config holds {} line(s) a {} script cannot carry:\n{}\n\nNothing was \
             written. Take the line out, mark it `@export=no`, or pass `--best-effort` to write \
             the script with each hole marked.",
            script.omitted.len(),
            format.as_str(),
            named.join("\n")
        ))
        .into());
    }

    let dir = std::path::PathBuf::from(out);
    let path = dir.join("bootstrap.sh");
    if path.exists() && !force {
        anyhow::bail!(
            "{} already exists. Move it aside or pass `--force`.",
            path.display()
        );
    }
    let (command, provenance) = match source {
        crate::app::bootstrap::Source::Declarations => (
            "--from-declaration",
            format!("the declarations in {}", state_origin(&app.config)),
        ),
        crate::app::bootstrap::Source::Scan => (
            "--from-scan",
            "what `shall adopt` found on this machine".to_string(),
        ),
    };
    let header = format!(
        "# Generated by `shall export {command} --format {}`\n\
         # Reproduces {provenance}. Needs a machine of the same OS.\n\
         # Shall is not needed to run it.\n",
        format.as_str()
    );
    let body = script.render(format, &header);
    tokio::fs::create_dir_all(&dir).await.ok();
    tokio::fs::write(&path, &body).await?;

    println!(
        "  wrote   {} ({} command{}{})",
        path.display(),
        script.steps.len(),
        if script.steps.len() == 1 { "" } else { "s" },
        if script.omitted.is_empty() {
            String::new()
        } else {
            format!(", {} hole(s) marked", script.omitted.len())
        }
    );
    if script.is_empty() && script.omitted.is_empty() {
        println!("  note    nothing to do: this config declares nothing installable");
    }
    // A best-effort artifact is made and is not complete, and the exit says which: a 0 here
    // would be a claim of completeness in the one number a script can read (II.94).
    if !script.omitted.is_empty() {
        return Err(crate::core::Error::Differences(format!(
            "wrote {} with {} line(s) this format cannot carry",
            path.display(),
            script.omitted.len()
        ))
        .into());
    }
    Ok(())
}

/// The declarations a bash artifact cannot reproduce, each with the sentence that says why.
///
/// **The rule is "the config holds the line", not "the plan needs it today"**, and the
/// conservative direction is deliberate: a line already satisfied on this machine would slip
/// past a plan-shaped check and then be missing from the script run on a fresh one — which is
/// exactly the machine this artifact is for.
fn unrepresentable(
    state: &crate::model::resolve::DesiredState,
) -> Vec<crate::app::bootstrap::Omission> {
    use crate::app::bootstrap::Omission;
    use crate::config::grammar::statement::Statement;
    let mut out = Vec::new();
    for (statement, origin) in &state.extras {
        if !statement.exports() {
            continue;
        }
        let why = match statement {
            // A secret is not in the config: `@decrypt` names ciphertext and a key Shall holds,
            // and a script carrying the plaintext would be the one thing an export must not do.
            // `@identity` is the same sentence about the key rather than the file.
            Statement::Link(_, options)
                if options.contains("decrypt") || options.contains("identity") =>
            {
                "the plaintext is not in the config, and an export must not carry it"
            }
            // The script is a file in the config repo, not a line in it.
            Statement::Exec(_, _) => {
                "the script it runs lives in the config repo, not in this artifact"
            }
            // Likewise every file of the tree.
            Statement::Dotfiles(_, _) => {
                "the folder it mirrors lives in the config repo, not in this artifact"
            }
            // Resource lines are not commands yet: each backend writes through its own adapter
            // table, and the exporter has no argv to offer for one.
            // TODO(comtai): export resource lines from the adapter tables instead of refusing.
            Statement::Link(_, _)
            | Statement::Service(_, _)
            | Statement::Setting(_, _)
            | Statement::Shim(_, _)
            | Statement::Schedule(_, _)
            | Statement::Repo { .. } => {
                "resource lines are not exported yet: this script carries package commands only"
            }
            _ => continue,
        };
        out.push(Omission {
            line: format!(
                "{}:{} {}",
                origin.file.display(),
                origin.line,
                statement.key()
            ),
            why: why.to_string(),
        });
    }
    out
}

/// Turn `backend -> specs` into commands, and report the backends that cannot answer.
///
/// **`None` is a refusal, not a skip** (II.94): a backend that installs by copying a file has no
/// argv to offer, and a script that quietly left its packages out would read as a complete
/// bootstrap for a machine that then came up without them.
fn emit_commands(
    app: &App,
    script: &mut crate::app::bootstrap::Script,
    by_backend: std::collections::BTreeMap<String, Vec<crate::core::PackageSpec>>,
    priority: crate::model::priority::Priority,
    omitted: &mut Vec<crate::app::bootstrap::Omission>,
) {
    // A bootstrap installs a compiler before a crate and a manager before its packages, which is
    // the order a sync uses and for the same reason (II.6, K1).
    let is_foundation = |b: &str| app.registry.get(b).map(|m| m.needs_root()).unwrap_or(false);
    let names: Vec<String> = by_backend.keys().cloned().collect();
    for backend in crate::app::rebuild::order_backends(&names, priority.order(), &is_foundation) {
        let Some(specs) = by_backend.get(&backend) else {
            continue;
        };
        let Some(caps) = app.registry.get(&backend) else {
            continue;
        };
        match caps
            .as_installable()
            .and_then(|installable| installable.planned_installs(specs))
        {
            Some(commands) => {
                for command in commands {
                    script.command(&command.binary, command.args, caps.sudo_for_write());
                }
            }
            None => omitted.push(crate::app::bootstrap::Omission {
                line: format!(
                    "{}: {}",
                    backend,
                    specs
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                why: format!(
                    "the `{backend}` backend cannot say which command installs these, so this \
                     script cannot either"
                ),
            }),
        }
    }
}

fn construct_spec(pkg: &crate::core::Package) -> crate::core::PackageSpec {
    crate::core::PackageSpec {
        name: pkg.name.clone(),
        backend: pkg.backend.clone(),
        ..Default::default()
    }
}

fn state_origin(config: &Config) -> String {
    config.config_root.display().to_string()
}

pub async fn handle_why(app: &App, package: &str, out: Output) -> Result<()> {
    // Q9: `why nosuchbackend:foo` reported it "not under Shall management" at exit 0 — true of
    // the string and useless, because the manager is the part that does not exist.
    app.resolver()
        .await
        .require_known_spec_backends(std::slice::from_ref(&package.to_string()))
        .await?;
    crate::app::insight::why(
        &app.config,
        &app.registry,
        &app.state,
        &app.executor,
        package,
        out,
    )
    .await
    .map_err(|e| e.into())
}

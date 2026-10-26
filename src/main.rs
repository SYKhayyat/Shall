use anyhow::{Context, Result};
use clap::Parser;
use shall::app::entrypoint::{expand_command_aliases, log_level_from_argv, plan_user_verb};
use shall::app::App;
use shall::cli::{Cli, Commands};
use shall::config::Config;
use shall::core::Output;
use std::env;
use tracing::warn;
use tracing_subscriber::EnvFilter;

// The dispatcher does reference every handler, so it globs them all — and that is a
// different relationship from the one `LX-11` was about. What was deleted is `verbs::prelude`
// re-exporting all nine into *each other*, which left no module boundary inside `verbs/` at
// all: 8,587 lines in one namespace stored in nine files, where moving a function between them
// was a no-op. The siblings now import each other by name (`grep "^use crate::verbs::"
// src/verbs/*.rs` is the map, and it is short), which is what makes a rule about where a
// handler belongs something a person can state and a compiler can check.
//
// This glob stays honest only while it is the dispatcher's. A *second* consumer is a sibling
// and should import by name.
//
// There are twelve modules now, not nine: `history` held fifteen handlers of which seven had
// nothing to do with history, so it became `history`, `ephemeral`, `inventory` and `portable`,
// and `adopt` went to `declare` where the other declaration-writing verbs are.
use shall::verbs::{
    check::*, cleanup::*, declare::*, ephemeral::*, history::*, inventory::*, packages::*, plan::*,
    portable::*, setup::*, sync::*, upgrade::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    // A closed output pipe (e.g. `shall search x | head`) makes `println!` fail with
    // EPIPE, which under `panic = "abort"` becomes a core dump ("Aborted"). Intercept
    // that one panic and exit quietly — the wanted output was already delivered. This
    // leaves SIGPIPE ignored for sockets, so network writes are unaffected.
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let is_broken_pipe = info.to_string().contains("Broken pipe")
            || info
                .payload()
                .downcast_ref::<String>()
                .is_some_and(|s| s.contains("Broken pipe"))
            || info
                .payload()
                .downcast_ref::<&str>()
                .is_some_and(|s| s.contains("Broken pipe"));
        if is_broken_pipe {
            std::process::exit(0);
        }
        default_panic_hook(info);
    }));

    // 1. Logging Initialization
    // Logs go to STDERR so that stdout carries only machine-readable payloads. Otherwise
    // `INFO` lines are interleaved with `--json` output on stdout, corrupting it for any
    // consumer (`shall search --json | jq`).
    //
    // The level is read straight off argv rather than off the parsed `Cli`, because this has
    // to be running before the shim hijack a few lines down — and reading it after clap is
    // exactly why `--verbose` used to do nothing at all.
    // A default run prints neither a timestamp nor a module path. `WARN
    // shall::verbs::packages` and an RFC3339 stamp are addressed to whoever is debugging
    // Shall, and the person reading them typed a package name — the sentence is for them, the
    // provenance is not. Both come back at `-v`, where somebody has asked for the internals.
    let argv: Vec<String> = std::env::args().collect();
    let level = log_level_from_argv(&argv);
    let filter = || EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let verbose = level.contains("debug") || level.contains("trace");
    // Asked, rather than left to `tracing-subscriber`'s default, which is colour-always. Without
    // this line `shall install nosuchpkg 2>&1 | grep` came back carrying escape codes and a run
    // redirected into a log file wrote them to disk. The question is about *stderr*, which is
    // where this writes — `utils::style::color_enabled` answers it for stdout, and a pipe on
    // stdout with a terminal on stderr is the ordinary arrangement, not an odd one.
    let ansi = shall::utils::style::color_enabled_on_stderr();
    if verbose {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter())
            .with_ansi(ansi)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter())
            .with_ansi(ansi)
            .without_time()
            .with_target(false)
            .init();
    }

    // 1.5 Where Shall's own data lives — before the shim hijack, which builds an `App` and so
    // reads it.
    settle_data_dir(&argv)?;

    // 2. Shim hijack
    if let Some(res) = attempt_shim_hijack().await? {
        return res;
    }

    // 3. CLI & Config Bootstrap
    // Expand user-defined command aliases (config `[command_aliases]`) BEFORE clap parses, so
    // an alias `up` can stand in for `upgrade --all`. Built-in subcommands always win.
    let raw_argv: Vec<String> = std::env::args().collect();
    let prefs = preferences_path_from_argv(&raw_argv)
        .and_then(|p| shall::config::Config::from_file(&p).ok());
    let aliases = prefs
        .as_ref()
        .map(|c| c.command_aliases.clone())
        .unwrap_or_default();
    let verbs = prefs.as_ref().map(|c| c.verbs.clone()).unwrap_or_default();

    // U35: a user-defined verb runs a *sequence* of built-in verbs. It is intercepted here,
    // before clap, because the verb name is not a Cli subcommand — clap would reject it. A verb
    // never shadows a built-in (built-ins always win), and every step must itself be a built-in
    // (composition only; arbitrary argv is U33's key, off by default).
    if !verbs.is_empty() {
        let known = known_subcommands();
        match plan_user_verb(&raw_argv, &verbs, &known) {
            Some(Ok(steps)) => return run_user_verb(steps).await,
            Some(Err(msg)) => {
                eprintln!("{}", msg);
                std::process::exit(shall::core::Exit::Failed.code());
            }
            None => {}
        }
    }

    let cli = if aliases.is_empty() {
        parse_or_exit(Cli::try_parse())
    } else {
        let known = known_subcommands();
        parse_or_exit(Cli::try_parse_from(expand_command_aliases(
            raw_argv, &aliases, &known,
        )))
    };
    // Before the config is read, because loading it can already run an external vars provider
    // — and a breakdown that starts after the first child has nothing to say about it.
    if cli.timings {
        shall::core::timing::enable();
    }

    let mut config = load_and_merge_config(&cli).await?;
    // T4: `watch` runs unattended, so nobody is present to touch a hardware key. Set on the
    // config BEFORE the registry is built, because the link backend takes an `Arc<Config>` at
    // construction and a touch-required `@decrypt` is skipped under this flag rather than
    // hanging the reconcile.
    if matches!(cli.command, Commands::Watch { .. }) {
        config.unattended = true;
    }
    apply_process_wide_config(&config);

    // The scheduled-run log is written by shall processes appending under systemd/launchd;
    // this is the one place every such process passes, so it is where the log gets its
    // rotation. Before anything else, so a run that fails two lines later still rotates.
    shall::app::scheduler::rotate_log_if_large();

    // 4. A hook fired by a manager that Shall itself is driving has nothing to add — the run
    //    that spawned it is already recording what it installed, and it holds the lock this
    //    process would wait two minutes for.
    if stands_down_inside_shall(&[&cli.command]) {
        return Ok(());
    }

    // 5. One writer at a time. Held for the whole run, released when `main` returns — a
    //    lock dropped before the last write is a lock over part of a set that must agree.
    let _data_lock = match acquire_data_lock(&[&cli.command]).await? {
        LockedRun::Proceed(lock) => lock,
        LockedRun::StandDown => return Ok(()),
    };

    // 6. Kernel Initialization
    let app = App::new(config).await?;

    // 7. Command Dispatcher (Modular A+ Routing)
    //
    // U21: the result is mapped to the exit-code table rather than returned straight, so a
    // guard refusal (3) and a read-only command that found work (2) are distinguishable from
    // a failure (1). `anyhow`'s default would collapse all three into 1.
    // Timed here, around the one dispatch, rather than inside each verb: a budget every verb
    // has to remember to check is a budget the next verb forgets. Nothing measured latency at
    // all before this, which is how a 98-second `info` shipped while `search` answered the same
    // question in seconds (E14).
    let started = std::time::Instant::now();
    let outcome = dispatch(&app, &cli).await;
    shall::core::latency::report_if_over(
        &shall::core::latency::subcommand_name(&cli.command),
        started.elapsed(),
    );
    // Before `finish`, which maps a refusal or a failure onto an exit code and leaves: a run
    // that ended badly is the one whose timing a user most wants to see.
    shall::core::timing::report(shall::core::timing::elapsed());
    finish(&app.config, outcome).await
}

/// Which of the four published codes a clap outcome is (Q3, II.8, V.92).
///
/// clap's own convention for a usage error is 2, and II.8 spends 2 on *a read-only command
/// looked and found work to do* — so a CI job branching on the documented table read a
/// mistyped subcommand as a drifted machine. A typo has not looked at the machine at all.
/// Asking for help or a version is an answer and stays 0.
fn clap_exit_code(kind: clap::error::ErrorKind) -> i32 {
    use clap::error::ErrorKind;
    match kind {
        // Asked for and answered.
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
        // `shall` with no subcommand. clap prints help as a courtesy and files it next to the
        // real thing, but nobody asked for help and no command ran — a script that reaches
        // here has a bug, and 0 would tell it everything is fine.
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => shall::core::Exit::Failed.code(),
        _ => shall::core::Exit::Failed.code(),
    }
}

/// Hand clap's own message to the user, then leave with Shall's code rather than clap's.
fn parse_or_exit(parsed: Result<Cli, clap::Error>) -> Cli {
    match parsed {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            std::process::exit(clap_exit_code(e.kind()));
        }
    }
}

/// Turn a command's result into this process's exit code (U21, `core::Exit`).
///
/// A refusal and a difference are printed as themselves — plainly, with no `Error:` prefix —
/// because neither is a malfunction. Only a real failure is reported as one.
pub(crate) async fn finish(config: &Config, outcome: Result<()>) -> Result<()> {
    use shall::core::Exit;
    match outcome {
        Ok(()) => Ok(()),
        Err(e) => {
            let code = match e.downcast_ref::<shall::core::Error>() {
                Some(shall::core::Error::Refused(msg)) => {
                    eprintln!("{}", msg);
                    // `on_guard_refusal` (XIII.13) fires here and nowhere else. Fired at this
                    // layer rather than inside the guard because announcing a refusal is a
                    // side effect, and a side effect inside a decision function runs wherever
                    // the decision is evaluated — tests included.
                    //
                    // This arm used to claim it was "the one point every refusal in the
                    // program passes through, so no command can be added that refuses without
                    // the hook hearing about it". That was false for nine sites — every
                    // security refusal, the whole SEC/T series — which returned 1 and were
                    // never announced. The claim is now checked rather than asserted:
                    // `tests/e2e/grader_refusal_exit_code_tests.rs` enumerates every site whose
                    // message says it is refusing and fails on one not built as
                    // `Error::Refused`, and fires a real hook through a real refusal. A
                    // sentence that quantifies over paths belongs in a test, not in a comment.
                    shall::app::events::EventHooks::load(config)
                        .fire(
                            shall::model::event::Event::OnGuardRefusal,
                            serde_json::json!({ "message": msg }),
                        )
                        .await;
                    Exit::Refused
                }
                Some(shall::core::Error::Differences(msg)) => {
                    if !msg.is_empty() {
                        eprintln!("{}", msg);
                    }
                    Exit::Differences
                }
                _ => {
                    // R-3's other half. Shall classifies every failure it can — a rate limit
                    // is `Transient` and says why — and nothing downstream could see the
                    // answer. The sweep harness therefore tested transience by RETRYING THE
                    // INSTALL IMMEDIATELY, which cannot succeed inside a 1236-second rate-limit
                    // window: it scored `defect`, the macOS leg went red, and the real-lifecycle
                    // ratchet fell 8 -> 7 and went red behind it. Two red CI jobs over a
                    // classification the program had already made.
                    //
                    // One stable line, on failure only, on stderr, in a shape a script can read
                    // without grepping an English sentence — the token is pinned by
                    // `tests/e2e/failure_class_line_tests.rs` precisely so the wording above it
                    // stays free to change.
                    print_failure_class(&e);
                    return Err(e);
                }
            };
            std::process::exit(code.code());
        }
    }
}

/// The one machine-readable line: what Shall thinks the failure it is about to report *is*.
///
/// `retryability()` already answers this and only two places consulted it. A caller that has to
/// re-derive the answer by running the command again is not reading the classification, it is
/// guessing at it — and an immediate retry is a guess that is wrong for exactly the failures the
/// classification gets right.
///
/// The vocabulary is `Retryability`'s own, so a variant added there and not handled here is a
/// compile error rather than a silently missing token.
fn print_failure_class(e: &anyhow::Error) {
    use shall::core::Retryability;
    use std::io::IsTerminal;

    // Addressed to a program, so it is written only where a program is listening. On a terminal
    // it was internal vocabulary on the first line of the first command a new user runs:
    //
    //     $ shall sync
    //     shall-failure-class: permanent
    //     Error: Configuration error: no `priority` file at …
    //
    // A pipe is exactly the condition under which both harnesses read it (G-6).
    if std::io::stderr().is_terminal() {
        return;
    }
    let class = match e
        .downcast_ref::<shall::core::Error>()
        .map(|x| x.retryability())
    {
        Some(Retryability::Transient) => "transient",
        Some(Retryability::Permanent) => "permanent",
        Some(Retryability::Exhausted) => "exhausted",
        // No Shall error at all, or one nothing classified: the same answer either way, and it
        // is the honest one — nobody looked.
        Some(Retryability::Unknown) | None => "unknown",
    };
    eprintln!("shall-failure-class: {class}");
}

/// The three `--json` flags whose help says "(requires --dry-run)", enforced.
///
/// **Clap cannot express this one, and the attempt is worse than the omission.** `requires` is
/// resolved against the subcommand's own arguments and `--dry-run` is global, so
/// `requires = "dry_run"` compiles, never fires for the case it is meant to catch, and turns
/// `sync --dry-run --json` — the documented, working combination — into a usage error with an
/// empty stdout. Measured: it broke the two tests that exist because a fleet reads that document
/// over SSH.
///
/// So the constraint lives here, where both halves are in scope. Without it, `shall sync --json`
/// printed the human summary or nothing at all and exited 0, and a script that forgot the pair
/// got a success code with no document to parse (B7).
///
/// `upgrade --json` is deliberately absent: it prints what `--security` actually remediated
/// *after* remediating it, so it is not a preview-only flag and its help no longer says it is.
fn refuse_json_without_dry_run(cli: &Cli) -> Result<()> {
    let verb = match &cli.command {
        Commands::Sync { json: true, .. } => "sync",
        Commands::Install { json: true, .. } => "install",
        Commands::Uninstall { json: true, .. } => "uninstall",
        _ => return Ok(()),
    };
    if cli.dry_run {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "`shall {verb} --json` writes the plan, so it only has one to write in a preview. Add \
         `--dry-run`.\n  Without it there is no document, and a script reading one would get a \
         success code and an empty answer."
    ))
}

pub(crate) async fn dispatch(app: &App, cli: &Cli) -> Result<()> {
    refuse_json_without_dry_run(cli)?;
    match &cli.command {
        Commands::Sync {
            locked,
            upgrade,
            json,
        } => {
            handle_sync(
                app,
                SyncMode {
                    locked: *locked,
                    upgrade: *upgrade,
                },
                Output::from_json_flag(*json),
            )
            .await
        }
        Commands::Watch {
            interval,
            on_change,
            pull,
            once,
        } => handle_watch(app, *interval, *on_change, *pull, *once).await,
        Commands::Upgrade {
            packages,
            backend,
            all,
            security,
            except,
            steps,
            no_steps,
            ignore_holds,
            ignore_pins,
            profile,
            module,
            json,
            canary,
            test,
        } => {
            handle_upgrade(
                app,
                UpgradeRequest {
                    packages,
                    backend: backend.as_deref(),
                    all: *all,
                    security: *security,
                    except,
                    ignore_holds: *ignore_holds,
                    ignore_pins: *ignore_pins,
                    profile,
                    module,
                    out: Output::from_json_flag(*json),
                    canary: *canary,
                    test,
                    // Neither flag is the default, and `overrides_with` makes the pair
                    // mutually exclusive rather than letting both be true at once.
                    steps: match (*steps, *no_steps) {
                        (true, _) => Some(true),
                        (_, true) => Some(false),
                        _ => None,
                    },
                },
            )
            .await
        }
        Commands::Install {
            packages,
            json,
            temp,
            into,
        } => {
            handle_install(
                app,
                packages,
                Output::from_json_flag(*json),
                temp.as_deref(),
                into.as_deref(),
            )
            .await
        }
        Commands::Uninstall {
            packages,
            json,
            temp,
            absent,
            purge: _,
        } => {
            handle_uninstall(
                app,
                packages,
                Output::from_json_flag(*json),
                temp.as_ref(),
                *absent,
            )
            .await
        }
        Commands::Shell { packages } => handle_shell(app.shell(), packages).await,
        Commands::Module(args) => {
            handle_module(&app.config, &app.resolver().await, &args.command).await
        }
        Commands::Schedule(args) => handle_schedule(app, &args.command).await,
        Commands::Snapshot(args) => {
            handle_snapshot(
                &app.config,
                &app.snapshot_manager,
                app.snapshot_restore(),
                &args.command,
            )
            .await
        }
        Commands::Rollback { reference } => handle_rollback(app, reference).await,
        Commands::Diff { from, to } => handle_diff(&app.vcs().manager(), from, to.as_deref()).await,
        Commands::Eval => handle_eval(&app.config, &app.registry).await,
        Commands::Repl => shall::app::repl::run(&app.config, &app.registry)
            .await
            .map_err(Into::into),
        Commands::Try { image } => handle_try(&app.config, &app.executor, image.as_deref()).await,
        Commands::Add {
            source,
            trust,
            force,
        } => handle_add(app, source, *trust, *force).await,
        Commands::Git(args) => handle_git(&app.vcs().manager(), &args.command).await,
        Commands::Repo(args) => handle_repo(app, &args.command).await,
        Commands::Search {
            query,
            json,
            installed,
        } => {
            handle_search(
                &app.inventory().await,
                &app.state,
                query,
                Output::from_json_flag(*json),
                *installed,
            )
            .await
        }
        Commands::Teleport { package, backend } => handle_teleport(app, package, backend).await,
        Commands::List {
            backend,
            json,
            outdated,
        } => {
            handle_list(
                app,
                backend.as_deref(),
                Output::from_json_flag(*json),
                *outdated,
            )
            .await
        }
        Commands::Info { package } => {
            handle_info(&app.inventory().await, &app.registry, package).await
        }
        Commands::RemoveOrphans => handle_remove_orphans(app).await,
        Commands::CleanCache { all } => handle_clean_cache(app, *all).await,
        Commands::Heal => handle_heal(app).await,
        Commands::Adopt {
            backends,
            enabled_only,
        } => handle_adopt(app, backends.clone(), *enabled_only).await,
        Commands::History => handle_history(app).await,
        Commands::Activate { profiles, add } => {
            handle_activate(app.profile_manager(), profiles, *add).await
        }
        Commands::Deactivate { profiles } => {
            handle_deactivate(app.profile_manager(), profiles).await
        }
        Commands::Profile(args) => handle_profile(app.profile_manager(), &args.command).await,
        Commands::Run {
            packages,
            command,
            args,
        } => handle_run(app.runner(), packages, command, args).await,
        Commands::Lock {
            what,
            names,
            except,
            list,
        } => {
            let selection = shall::core::lock_kind::LockSelection::parse(what, except)?
                .narrowed_by_config(&app.config.lock.freezes());
            handle_lock(app, &selection, names, *list).await
        }
        Commands::Unlock {
            what,
            names,
            except,
            list,
        } => {
            // **`[lock] freeze` does not narrow `unlock`.** It says what a machine freezes by
            // default, and reading it as "what a machine may release" would make an entry
            // recorded before the preference changed permanently unreleasable.
            let selection = shall::core::lock_kind::LockSelection::parse(what, except)?;
            handle_unlock(
                &app.config,
                &app.registry,
                &app.resolver().await,
                &selection,
                names,
                *list,
            )
            .await
        }
        Commands::Plan { out } => handle_plan(app, out).await,
        Commands::Apply { plan, yes } => handle_apply(app, plan, *yes).await,
        Commands::Update => handle_update(&app.managers().await).await,
        Commands::Reset { force } => handle_reset(&app.config, &app.state, *force).await,
        Commands::Check { section, json } => {
            handle_check(app, section.as_deref(), Output::from_json_flag(*json)).await
        }
        Commands::Vars => handle_vars(&app.config, &app.registry).await,
        Commands::PurgeUndeclared { allow_mass_purge } => {
            handle_purge_undeclared(app, *allow_mass_purge).await
        }
        Commands::Adapters { surface, json } => {
            handle_adapters(
                &app.config,
                surface.as_deref(),
                Output::from_json_flag(*json),
            )
            .await
        }
        Commands::Protected { packages, json } => {
            handle_protected(
                &app.config,
                &app.registry,
                &app.resolver().await,
                packages,
                Output::from_json_flag(*json),
            )
            .await
        }
        Commands::Unmanage { packages, json } => {
            handle_unmanage(app, packages, Output::from_json_flag(*json)).await
        }
        Commands::Rebuild {
            packages,
            backend,
            all,
        } => handle_rebuild(app, packages, backend.as_deref(), *all).await,
        Commands::Config(args) => handle_config(&app.config, &args.command).await,
        Commands::Path { explain, set } => handle_path(cli, *explain, set.as_deref()).await,
        Commands::Edit { file } => handle_edit(cli, file.as_deref()).await,
        Commands::Init { force, interactive } => handle_init(app, *force, *interactive).await,
        Commands::Sbom => handle_sbom(&app.config, &app.registry, &app.state).await,
        Commands::Export {
            format,
            out,
            stdout,
            force,
        } => {
            handle_export(
                &app.config,
                &app.registry,
                &app.state,
                format.as_deref(),
                out,
                *stdout,
                *force,
            )
            .await
        }
        Commands::Bundle {
            out,
            artifacts,
            archive,
        } => handle_bundle(app, out, *artifacts, *archive).await,
        Commands::Restore { dir, force } => {
            handle_restore(&app.config, &app.state, dir, *force).await
        }
        Commands::Why { package, json } => {
            handle_why(app, package, Output::from_json_flag(*json)).await
        }
        Commands::Service(args) => handle_service(app, &args.command).await,
        Commands::Bisect { test, yes } => {
            shall::app::bisect::bisect(&app.config, &app.snapshot_manager, test, *yes)
                .await
                .map_err(|e| e.into())
        }
        Commands::Fleet(args) => {
            shall::app::fleet::fleet(&app.config, &args.hosts, args.sync, args.apply)
                .await
                .map_err(|e| e.into())
        }
        Commands::Hooks(args) => handle_hooks(&app.registry, &args.command).await,
        Commands::HookRecord {
            manager,
            op,
            targets,
        } => {
            handle_hook_record(
                &app.declarations(),
                &app.state,
                &app.vcs(),
                manager,
                op,
                targets,
            )
            .await
        }
        Commands::HookReconcile { manager } => {
            handle_hook_reconcile(&app.registry, &app.state, &app.vcs(), manager).await
        }
        Commands::HookObserve {
            manager,
            learn,
            argv,
        } => handle_hook_observe(app, manager.as_deref(), *learn, argv).await,
        Commands::Hold { packages } => {
            handle_hold(
                app.holds().await,
                &app.resolver().await,
                &app.state,
                packages,
            )
            .await
        }
        Commands::Unhold { packages } => {
            handle_unhold(&app.resolver().await, &app.state, packages).await
        }
        Commands::Policy => handle_policy(app).await,
        Commands::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            shall::cli::generate_completions(*shell, &mut cmd);
            Ok(())
        }
        Commands::SelfUpgrade { git, check } => handle_self_upgrade(git.as_deref(), *check).await,
    }
}

/// Repository a `self-upgrade` installs from: explicit `--git`, else `$SHALL_REPO`, else the
/// upstream default (kept in sync with `scripts/install.sh`).
pub(crate) fn self_upgrade_repo(git: Option<&str>) -> String {
    git.map(|s| s.to_string())
        .or_else(|| std::env::var("SHALL_REPO").ok())
        .unwrap_or_else(|| "https://github.com/SYKhayyat/Shall".to_string())
}

pub(crate) async fn cargo_install_from(
    repo: &str,
    locked: bool,
) -> std::io::Result<std::process::ExitStatus> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("install").arg("--git").arg(repo).arg("--force");
    if locked {
        cmd.arg("--locked");
    }
    // The terminal-handoff door: a `cargo install --git` compiles for minutes and the person is
    // reading its progress, so it is inherited and unbounded — but owned, because a compile left
    // running after Shall has gone still writes to `~/.cargo/bin` when it finishes.
    shall::core::supervise::supervised_status(cmd, "cargo install")
        .await
        .map_err(std::io::Error::other)
}

pub(crate) async fn handle_self_upgrade(git: Option<&str>, check: bool) -> Result<()> {
    let repo = self_upgrade_repo(git);
    println!("Current version : shall {}", shall::VERSION);
    if check {
        println!("Upgrade source  : {}", repo);
        println!("Run `shall self-upgrade` to rebuild and install the latest from source.");
        return Ok(());
    }
    if which::which("cargo").is_err() {
        anyhow::bail!(
            "`cargo` (the Rust toolchain) is required to self-upgrade. Install it from \
             https://rustup.rs, or re-run the Shall install script."
        );
    }
    println!("Rebuilding shall from {repo} via cargo — this can take a few minutes...");
    // Reproducible build first (--locked); fall back to a loose build, exactly like install.sh.
    let first = cargo_install_from(&repo, true).await;
    let ok = matches!(&first, Ok(s) if s.success());
    if !ok {
        warn!("locked build failed; retrying without --locked...");
        let second = cargo_install_from(&repo, false)
            .await
            .context("running `cargo install`")?;
        if !second.success() {
            anyhow::bail!("cargo install failed; Shall was not upgraded.");
        }
    }
    println!("Done. Run `shall --version` to confirm the new build.");
    Ok(())
}

/// The value of a `--flag VALUE` / `--flag=VALUE` in raw argv.
///
/// Command aliases are expanded before clap runs, so this pre-parse cannot ask clap where the
/// repo is. It peeks at the flags and hands them to the same resolver the app uses — peeking
/// is unavoidable here, resolving a second time is not.
pub(crate) fn flag_from_argv(argv: &[String], names: &[&str]) -> Option<String> {
    let mut it = argv.iter();
    while let Some(a) = it.next() {
        if names.contains(&a.as_str()) {
            return it.next().cloned();
        }
        for n in names {
            if let Some(rest) = a.strip_prefix(&format!("{}=", n)) {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// `--data-dir`, and the two environment variables, settled once before anything reads them.
///
/// **The flag sets the variable rather than becoming a second answer.** Six places ask "where is
/// Shall's data" — `safe_data_dir`, `Layout::from_env`, `StateRegistry::load_default`, the
/// config default, the rehearsal sandbox, the test fixtures — and every one of them reads
/// `$SHALL_DATA_DIR`. A flag threaded through as a separate value would have to reach all six,
/// and the one it missed would be the one that wrote to the developer's real registry. Config
/// got a first-class flag and state got an undocumented variable, and that asymmetry is what
/// turned `--config-dir` from a testing affordance into a trap (AU4): a fresh sandbox planned
/// seven removals against the real machine's managed state.
///
/// Read from argv rather than from the parsed `Cli` because the shim hijack builds an `App`
/// before clap runs — the same reason the log level is read here.
///
/// Both variables are checked for absoluteness at this one point, because the readers above
/// return a `PathBuf` and cannot refuse anything (AU2).
fn settle_data_dir(argv: &[String]) -> Result<()> {
    use shall::config::settings::absolute_or_refuse;

    if let Some(flag) = flag_from_argv(argv, &["--data-dir"]) {
        let dir = absolute_or_refuse(std::path::PathBuf::from(flag), "`--data-dir`")?;
        std::env::set_var("SHALL_DATA_DIR", dir);
    } else if let Some(dir) = std::env::var_os("SHALL_DATA_DIR").filter(|v| !v.is_empty()) {
        absolute_or_refuse(std::path::PathBuf::from(dir), "`$SHALL_DATA_DIR`")?;
    }
    if let Some(dir) = std::env::var_os("SHALL_CONFIG_DIR").filter(|v| !v.is_empty()) {
        absolute_or_refuse(std::path::PathBuf::from(dir), "`$SHALL_CONFIG_DIR`")?;
    }
    Ok(())
}

/// Where `preferences.toml` is, for the pre-clap alias load.
///
/// `--config` names the file; otherwise `locate` answers with `--config-dir`,
/// `$SHALL_CONFIG_DIR`, the settings file, then the default — the one resolution, so the
/// aliases come out of the file the rest of the run will read (X.6).
pub(crate) fn preferences_path_from_argv(argv: &[String]) -> Option<std::path::PathBuf> {
    if let Some(p) = flag_from_argv(argv, &["-c", "--config"]) {
        return Some(std::path::PathBuf::from(p));
    }
    let dir = flag_from_argv(argv, &["--config-dir"]).map(std::path::PathBuf::from);
    shall::app::locate::locate(dir.as_deref())
        .ok()
        .map(|r| r.path.join(shall::config::PREFERENCES_FILE_NAME))
}

/// Does this invocation consist of hooks fired by a manager Shall is already driving?
///
/// **Both doors, one rule.** The stand-down lived inline in `main` and covered a single
/// subcommand; `run_user_verb` is the other way a `Commands` reaches a lock, and a `[verbs]`
/// entry may name `hook-record` as a step because `plan_user_verb` admits any built-in. A guard
/// on one entry point is a guard on nothing, which is the shape `CLAUDE.md` names and this
/// finding is the third instance of.
///
/// `any`, not `all`: one hook step is enough to make the whole verb wait on a lock this
/// process's parent is holding, and a sequence that deadlocks halfway is not better than one
/// that stands down.
pub(crate) fn stands_down_inside_shall(commands: &[&Commands]) -> bool {
    env::var_os(shall::core::executor::INSIDE_SHALL).is_some()
        && commands.iter().any(|c| c.is_manager_hook())
}

/// What taking the lock decided about whether this run happens at all.
pub(crate) enum LockedRun {
    /// The lock is held, or the command needs none. The run proceeds.
    Proceed(Option<shall::core::datalock::DataLock>),
    /// A manager hook found the directory locked, and stood down rather than waiting.
    StandDown,
}

/// Take the lock for a mutating command, asking the command itself.
///
/// It used to be read from argv and matched against a hand-written list of twenty-one names,
/// on the reasoning that a subcommand added later would then be locked by default rather than
/// forgotten by a match arm. The list was the thing that rotted — twelve of its entries once
/// named commands the program did not have, `history` was on it while reaching the whole
/// install path, and `fleet` was off it while touching nothing local. `Commands::writes` is
/// exhaustive, so a subcommand added later does not compile until it answers, which is the
/// property the argv read was reaching for and could not have.
///
/// **A `hook-*` subcommand never waits for the lock, marker or no marker.** The environment
/// stand-down above cannot be relied on to fire: `SHALL_INSIDE` is set on the child
/// environment, and every manager `pm_hooks` targets — apt, dnf, zypper — is `needs_root`, so
/// the argv goes through `sudo`, whose `env_reset` rebuilds the environment and keeps only
/// `env_keep`. The marker is not in that set, so on the ordinary configuration — a normal
/// user on a normal Linux desktop — the hook arrives with no marker and waits out the full
/// 120 seconds for a lock its own grandparent holds and will not release until the sync ends.
///
/// Depending on the environment surviving `sudo` is the mistake; not depending on it is the
/// fix. A hook that finds the directory locked has nothing a wait can buy it — by the time it
/// won, the run holding the lock has finished and recorded what the hook was going to. That
/// also covers the case the marker is legitimately absent for: an `apt install` a person typed
/// while a `shall sync` runs, where the stand-down should not fire and the wait still costs
/// two minutes.
pub(crate) async fn acquire_data_lock(commands: &[&Commands]) -> Result<LockedRun> {
    let Some(writer) = commands.iter().find(|c| c.writes()) else {
        return Ok(LockedRun::Proceed(None));
    };
    let name = shall::core::latency::subcommand_name(writer);

    // `any`, not the writer alone, for the reason `stands_down_inside_shall` gives: one hook
    // step is enough to make the whole sequence wait on a lock this process's parent holds.
    if commands.iter().any(|c| c.is_manager_hook()) {
        return Ok(
            match shall::core::datalock::DataLock::try_for_one_step(&name)? {
                Some(lock) => LockedRun::Proceed(Some(lock)),
                None => LockedRun::StandDown,
            },
        );
    }

    let lock = shall::core::datalock::DataLock::for_one_step(&name).await?;
    Ok(LockedRun::Proceed(Some(lock)))
}

pub(crate) fn known_subcommands() -> std::collections::HashSet<String> {
    <Cli as clap::CommandFactory>::command()
        .get_subcommands()
        .flat_map(|s| {
            std::iter::once(s.get_name().to_string())
                .chain(s.get_all_aliases().map(|a| a.to_string()))
        })
        .collect()
}

/// Seed the settings that live in process-wide cells rather than in `App`.
///
/// One function because there are two entry points that load a config, and a setting wired
/// into one of them is a setting that does nothing under `run_user_verb`.
fn apply_process_wide_config(config: &shall::config::Config) {
    shall::backends::node_registry::set_http_timeout(config.network_timeout_secs);
    shall::core::executor::set_command_idle_timeout(config.command_idle_timeout_secs);
    shall::core::executor::set_query_bounds(
        config.query_idle_timeout_secs,
        config.read_retry_attempts,
    );
    shall::core::executor::set_sudo_password_timeout(config.sudo_password_timeout_secs);
    shall::core::download::set_max_download_bytes(config.max_download_bytes);
    shall::utils::archive::set_max_unpacked_bytes(config.max_unpacked_bytes);
}

/// Run a user verb: build the config and app once from the shared leading flags, then dispatch
/// each step against them in order, stopping at the first failure.
///
/// **One data lock covers the whole verb, and it is taken when ANY step writes.** The verb name
/// is not a subcommand, so this used to lock unconditionally as the safe default for a sequence
/// that may install or remove; now that each step parses to a `Commands`, the sequence can be
/// asked instead. A verb of five readers stops holding the writer lock, and a verb whose third
/// step syncs takes it before the first step runs rather than partway through.
pub(crate) async fn run_user_verb(steps: Vec<Vec<String>>) -> Result<()> {
    // `plan_user_verb` validates every step before this runs; a caller that arrives without
    // it must not fall back to clap's own error exit, whose code means something else here.
    let parsed: Vec<Cli> = steps
        .iter()
        .map(|s| match Cli::try_parse_from(s) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!(
                    "the verb step `{}` is not a valid command line: {e}",
                    s[1..].join(" ")
                );
                std::process::exit(shall::core::Exit::Failed.code());
            }
        })
        .collect();
    if stands_down_inside_shall(&parsed.iter().map(|c| &c.command).collect::<Vec<_>>()) {
        return Ok(());
    }
    let config = load_and_merge_config(&parsed[0]).await?;
    apply_process_wide_config(&config);
    // The lock spans the whole verb, so the question is whether any step writes — not whether
    // the first one does. Taking it per step would release it between two commands that have
    // to agree about the same registry.
    let commands: Vec<&Commands> = parsed.iter().map(|c| &c.command).collect();
    let _data_lock = match acquire_data_lock(&commands).await? {
        LockedRun::Proceed(lock) => lock,
        LockedRun::StandDown => return Ok(()),
    };
    let app = App::new(config).await?;
    for cli in &parsed {
        let outcome = dispatch(&app, cli).await;
        if outcome.is_err() {
            return finish(&app.config, outcome).await;
        }
    }
    finish(&app.config, Ok(())).await
}

// ============================================================================
// KERNEL HELPERS
// ============================================================================

pub(crate) async fn attempt_shim_hijack() -> Result<Option<Result<()>>> {
    let current_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "shall".to_string());
    // **The `.exe` never reaches the matcher.** Shim declarations are keyed on bare names
    // (`shim:jq`), so a Windows copy named `jq.exe` used to miss the ledger, degrade the spec
    // to the literal string `jq.exe`, and — per run.rs's respawn note — resolve back to this
    // same file: the unbounded spawn chain. Case-insensitive both ways, because `SHALL.EXE`
    // and `Shall.exe` are this binary too.
    let bare = current_name
        .strip_suffix(std::env::consts::EXE_SUFFIX)
        .unwrap_or(&current_name)
        .to_lowercase();
    if bare != "shall" {
        let root = shall::app::locate::locate(None)?.path;
        let config =
            shall::config::Config::from_file(&root.join(shall::config::PREFERENCES_FILE_NAME))
                .unwrap_or_default();
        let app = App::new(config).await?;
        return Ok(Some(
            app.runner()
                .exec_shim(&bare, &env::args().collect::<Vec<_>>()[1..])
                .await
                .map_err(|e| e.into()),
        ));
    }
    Ok(None)
}

pub(crate) async fn load_and_merge_config(cli: &Cli) -> Result<shall::config::Config> {
    // Where the repo is: --config-dir, then $SHALL_CONFIG_DIR, then Shall's settings file,
    // then the default. This has to resolve BEFORE `preferences.toml` is opened, because
    // that file lives inside the root it would otherwise have to announce.
    let located = shall::app::locate::locate(cli.config_dir.as_deref())?;
    let path = cli
        .config
        .clone()
        .unwrap_or_else(|| located.path.join(shall::config::PREFERENCES_FILE_NAME));
    let mut config =
        tokio::task::spawn_blocking(move || shall::config::Config::from_file(&path)).await??;
    config.config_root = located.path;
    config.merge_cli_overrides(shall::config::CliOverrides {
        dry_run: cli.dry_run,
        yes: cli.yes,
        verbose: cli.verbose > 0,
        allow_mass_removal: cli.allow_mass_removal,
        allow_mass_install: cli.allow_mass_install,
        config_path: None,
    });
    // The one place `--dry-run` becomes a property of the process. Set after the config merge
    // so a `dry_run = true` in `preferences.toml` counts too, and before dispatch so no write
    // can run ahead of it. Every config write consults this instead of each verb remembering
    // to — which five verbs did not (`activate`, `deactivate`, `lock`, `git init`,
    // `config init`), and `--dry-run activate Work` left you on Work without printing a line.
    shall::core::dry_run::set(config.dry_run);

    // A per-run acknowledgement, never a config key (U23): a machine that always bypasses the
    // dotfiles collision check is a machine where the check does not exist.
    if cli.replace_existing {
        config.replace_existing = true;
    }
    // --quiet has no config-file merge counterpart; apply it directly (a set flag wins).
    if cli.quiet {
        config.quiet = true;
    }
    // `--no-cache` is the whole off-switch: the TTL is what the disk layer is built from, so
    // zeroing it here means nothing downstream has a second way to be on. The same zero is
    // what keeps a cached listing out of every command that writes its answer down
    // (`cache_may_answer`) — the setting says how long a reading may be reused, never that a
    // plan or an adoption may be built on one.
    if cli.no_cache
        || !shall::core::installed::InstalledListings::cache_may_answer(
            &shall::core::latency::subcommand_name(&cli.command),
        )
    {
        config.installed_cache_secs = 0;
    }
    // `uninstall --purge` is a `[remove] purge` for this run only. Read here, where CLI flags
    // become config, because `config` is shared read-only by the time a command runs.
    if let Commands::Uninstall { purge: true, .. } = cli.command {
        config.purge_this_run = true;
    }
    // `--keep-going` is per-run by construction: there is no file key to read it from.
    if cli.keep_going {
        config.keep_going_this_run = true;
    }
    // --no-progress is the real off-switch for the progress indicators (S5). A set flag wins
    // over the `show_progress` config default.
    if cli.no_progress {
        config.show_progress = false;
    }
    Ok(config)
}

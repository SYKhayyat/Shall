// src/app/fleet.rs
//
// `fleet` — compare many machines over SSH against their manifests and report drift,
// optionally reconciling each with `shall sync`. It assumes `shall` is installed on the
// remote hosts and SSH is configured non-interactively (keys/agent). Remote invocations
// are read-only unless you pass the flags that opt into changes.
//
// There is no `clone` command. It was removed, implementation and all, because copying the
// installed set without the intent produces a machine nobody can explain; `git clone` of the
// manifests plus `shall sync` is the supported path. Do not reintroduce it here.

use crate::config::Config;
use crate::core::{Error, Result};
use serde_json::{json, Value};
use tracing::{info, warn};

/// Reject a host `ssh` would read as an option. A value like `-oProxyCommand=…` runs a command
/// on THIS machine, not the remote one.
pub fn check_host(host: &str) -> Result<()> {
    if host.starts_with('-') {
        return Err(Error::Config(format!(
            "`{}` is not a host name — a host cannot begin with `-`, because ssh would read it \
             as an option and run a command on this machine instead of the remote one.",
            host
        )));
    }
    Ok(())
}

/// Run a command on a remote host over SSH and return its stdout.
async fn ssh_capture(host: &str, remote_cmd: &str) -> Result<String> {
    check_host(host)?;
    // `-o BatchMode=yes` fails fast instead of hanging on a password prompt. `--` must follow
    // it, not precede it, or ssh stops reading the `-o` pair as options.
    let mut command = tokio::process::Command::new("ssh");
    command
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("--")
        .arg(host)
        .arg(remote_cmd);
    // Supervised: `BatchMode` stops ssh asking for a password, but nothing stopped it hanging on
    // a host that accepts the connection and never answers — and a fleet query abandoned midway
    // left one ssh per host behind it.
    let out = crate::core::supervise::supervised_output(command, "ssh", false)
        .await
        .map_err(|e| Error::Other(format!("failed to launch ssh for {}: {}", host, e)))?;
    if !out.status.success() {
        return Err(Error::Other(format!(
            "ssh {} `{}` failed: {}",
            host,
            remote_cmd,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
        &out.stdout,
    )))
}

/// What the remote runs. `check` is the read-only "what is going on here" command, and its
/// `--json` form is the only output of Shall's that is a document rather than a report: it
/// prints the array below and returns, so nothing else lands on the remote's stdout.
///
/// It used to ask for a `status --json` that no longer exists — the ten looking-commands
/// collapsed into `check` and the old names were deleted rather than aliased. So every host
/// answered "unrecognized subcommand" with exit 2, every row read ERROR, and `shall fleet`
/// could not report a correctly installed machine as in sync. Nothing caught it, because the
/// gate that compares invocations to the clap surface was drawn around `args.rs`;
/// `tests/review/named_commands_exist_tests.rs` is drawn around the property instead.
pub const REMOTE_CHECK: &str = "shall check --json";

/// What the remote runs to converge. Named beside its twin so the two cannot drift apart.
pub const REMOTE_SYNC: &str = "shall sync -y";

/// Per-host drift summary, read from a remote `shall check --json`.
#[derive(Debug)]
pub struct HostDrift {
    pub host: String,
    pub to_install: usize,
    pub to_remove: usize,
    pub unmanaged: usize,
    /// The drift section's own verdict, which is wider than the two counts beside it: a machine
    /// whose packages match but whose `link:` tree does not has drifted, and reporting it as in
    /// sync would be the fleet agreeing with a machine that disagrees with its files.
    pub drifted: bool,
    pub error: Option<String>,
}

impl HostDrift {
    pub fn in_sync(&self) -> bool {
        self.error.is_none() && !self.drifted
    }
}

/// What one host said about itself.
pub struct Reading {
    pub to_install: usize,
    pub to_remove: usize,
    pub unmanaged: usize,
    pub drifted: bool,
}

/// Read a remote `shall check --json` document. Pure — unit tested.
///
/// The counts come from each section's `counts` object, never from its `summary` sentence.
/// Those numbers are in the document precisely so that a fleet does not have to make an API
/// out of somebody's phrasing.
pub fn parse_check(json: &str) -> Result<Reading> {
    let v: Value = serde_json::from_str(json).map_err(|e| Error::Json(e.to_string()))?;
    let Some(sections) = v.as_array() else {
        return Err(Error::Json(
            "`shall check --json` returns an array of sections; this host returned something \
             else. Is `shall` on its PATH, and the same version?"
                .into(),
        ));
    };
    let section = |name: &str| {
        sections
            .iter()
            .find(|s| s.get("section") == Some(&json!(name)))
    };
    let count = |s: Option<&Value>, key: &str| -> usize {
        s.and_then(|s| s.get("counts"))
            .and_then(|c| c.get(key))
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as usize
    };

    // A document with no drift section is not a check report, and guessing "in sync" from its
    // absence is the one wrong answer a fleet must never give.
    let Some(drift) = section("drift") else {
        return Err(Error::Json(
            "`shall check --json` returned no `drift` section, so this host's answer says \
             nothing about whether it matches its manifests"
                .into(),
        ));
    };

    Ok(Reading {
        to_install: count(Some(drift), "install"),
        to_remove: count(Some(drift), "remove"),
        unmanaged: count(section("unmanaged"), "unmanaged"),
        drifted: drift.get("ok").and_then(|b| b.as_bool()) != Some(true),
    })
}

/// Query each host's drift versus its manifests and report; optionally reconcile.
/// `do_sync` reconciles only the DRIFTED machines; `do_apply` runs `shall sync -y` on EVERY
/// reachable host regardless of drift (a deliberate fleet-wide push).
pub async fn fleet(config: &Config, hosts: &[String], do_sync: bool, do_apply: bool) -> Result<()> {
    let hosts: Vec<String> = if hosts.is_empty() {
        config.fleet_hosts.clone()
    } else {
        hosts.to_vec()
    };
    if hosts.is_empty() {
        return Err(Error::Config(
            "no hosts given and `fleet_hosts` is empty in preferences.toml".into(),
        ));
    }
    for host in &hosts {
        check_host(host)?;
    }

    // A fleet tool's whole subject is N machines, and it used to talk to them one at a time:
    // every host paid the full SSH handshake plus the remote command's runtime, added end to
    // end. Ten hosts at 3s each was 30s of waiting for answers that have nothing to say to one
    // another. Ordered, so the table below still reads in the order the user listed them.
    use futures::stream::StreamExt;
    let report: Vec<HostDrift> = futures::stream::iter(hosts.iter().cloned())
        .map(|host| async move {
            let read = match ssh_capture(&host, REMOTE_CHECK).await {
                Ok(json) => parse_check(&json),
                Err(e) => Err(e),
            };
            match read {
                Ok(r) => HostDrift {
                    host,
                    to_install: r.to_install,
                    to_remove: r.to_remove,
                    unmanaged: r.unmanaged,
                    drifted: r.drifted,
                    error: None,
                },
                // A host that could not answer is not a host that answered "in sync". `in_sync`
                // already refuses on the error; `drifted` agrees with it rather than sitting at
                // a default that would read as converged if the two ever came apart.
                Err(e) => HostDrift {
                    host,
                    to_install: 0,
                    to_remove: 0,
                    unmanaged: 0,
                    drifted: true,
                    error: Some(e.to_string()),
                },
            }
        })
        .buffered(config.network_parallel.max(1))
        .collect()
        .await;

    let in_sync = report.iter().filter(|h| h.in_sync()).count();
    println!(
        "{} of {} machine(s) match their manifests.\n",
        in_sync,
        report.len()
    );
    println!(
        "{:<28} {:>9} {:>8} {:>10}  STATUS",
        "HOST", "INSTALL", "REMOVE", "UNMANAGED"
    );
    for h in &report {
        if let Some(err) = &h.error {
            println!(
                "{:<28} {:>9} {:>8} {:>10}  ERROR: {}",
                h.host, "-", "-", "-", err
            );
        } else {
            let status = if h.in_sync() { "in sync" } else { "DRIFT" };
            println!(
                "{:<28} {:>9} {:>8} {:>10}  {}",
                h.host, h.to_install, h.to_remove, h.unmanaged, status
            );
        }
    }

    // Reconciliation. `--apply` pushes to every reachable host; `--sync` touches only drift.
    if do_apply || do_sync {
        let targets: Vec<&HostDrift> = if do_apply {
            println!("\nApplying `{}` to all reachable machines ...", REMOTE_SYNC);
            report.iter().filter(|h| h.error.is_none()).collect()
        } else {
            println!("\nReconciling drifted machines with `{}` ...", REMOTE_SYNC);
            report
                .iter()
                .filter(|h| !h.in_sync() && h.error.is_none())
                .collect()
        };
        if targets.is_empty() {
            println!("  (nothing to do)");
        }
        // Concurrent for the same reason the drift pass is: these are separate machines, each
        // running its own sync, contending for nothing. Serial, a fleet-wide push cost the sum
        // of every host's sync.
        let outcomes: Vec<(String, std::result::Result<(), String>)> =
            futures::stream::iter(targets)
                .map(|h| async move {
                    info!("syncing {} ...", h.host);
                    let outcome = ssh_capture(&h.host, REMOTE_SYNC)
                        .await
                        .map(|_| ())
                        .map_err(|e| e.to_string());
                    (h.host.clone(), outcome)
                })
                .buffered(config.network_parallel.max(1))
                .collect()
                .await;

        let mut ok = 0usize;
        let mut failed_hosts: Vec<String> = Vec::new();
        for (host, outcome) in outcomes {
            match outcome {
                Ok(()) => {
                    println!("  {} synced.", host);
                    ok += 1;
                }
                Err(e) => {
                    warn!("sync failed on {}: {}", host, e);
                    failed_hosts.push(host);
                }
            }
        }
        println!(
            "\nApplied to {} host(s), {} failed.",
            ok,
            failed_hosts.len()
        );
        // **The exit code is the interface here.** A fleet rollout's wrapper reads nothing
        // but the code, and `--apply` reporting "3 failed" at exit 0 was green over a fleet
        // where every host failed — the same lie `--keep-going` was caught telling (B1).
        if !failed_hosts.is_empty() {
            return Err(crate::core::Error::Other(format!(
                "{} host(s) failed to sync: {}",
                failed_hosts.len(),
                failed_hosts.join(", ")
            )));
        }
    }
    Ok(())
}

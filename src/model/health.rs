//! Health checks, and what a failing one means (XIII.5, U7).
//!
//! **Two scopes, one revert path (U7, ruled 2026-07-24).** `@health=` on a line answers *did
//! this upgrade break this*; a `health` list in `preferences.toml` catches what a package
//! cannot see — the boot, the network, the thing two packages away. They are not alternatives,
//! and the machine does not care which kind noticed: a failure restores the snapshot either way.
//!
//! **The precondition is the feature.** A health check that cannot revert is a health check
//! that tells you the machine is broken and leaves it broken. So the absence of a snapshot
//! provider is decided *before* anything is installed, not discovered afterwards when the
//! reverting would have mattered.
//!
//! Pure: parsing a probe, and deciding what a set of results means. Running a command and
//! restoring a snapshot belong to the caller.

/// One thing to check after a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    /// `port:8080` — succeeds when a TCP connection to localhost opens.
    Port(u16),
    /// `cmd:systemctl is-active nginx`, or a bare command. Succeeds on exit 0.
    Command(String),
}

impl Probe {
    /// Parse a probe as written on a line or in `preferences.toml`.
    ///
    /// A bare string is a command, because that is what people write. `port:` is the one
    /// prefix, because "is something listening" is the check nobody can spell portably as a
    /// command — `nc`, `ss` and `Test-NetConnection` are three answers to one question.
    pub fn parse(written: &str) -> Option<Probe> {
        let written = written.trim();
        if written.is_empty() {
            return None;
        }
        if let Some(port) = written.strip_prefix("port:") {
            // A malformed port is not silently a command named "port:donkey": the line says
            // port, so it is a port or it is an error the caller reports.
            return port.trim().parse().ok().map(Probe::Port);
        }
        let cmd = written.strip_prefix("cmd:").unwrap_or(written).trim();
        if cmd.is_empty() {
            return None;
        }
        Some(Probe::Command(cmd.to_string()))
    }
}

impl std::fmt::Display for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Probe::Port(p) => write!(f, "port:{}", p),
            Probe::Command(c) => f.write_str(c),
        }
    }
}

/// A probe and what it belongs to, so a failure can name the thing that broke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// `apt:nginx` for a line's check, or `preferences.toml` for a machine-wide one.
    pub subject: String,
    pub probe: Probe,
}

/// What to do once the probes have run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Everything passed, or nothing was declared.
    Healthy,
    /// Something failed and there is a snapshot to go back to.
    Revert {
        failed: Vec<String>,
        snapshot: String,
    },
    /// Something failed and there is nothing to revert to. The machine is left as it is —
    /// Shall will not pretend it fixed anything — and the failure is loud.
    FailedWithoutRevert { failed: Vec<String> },
}

impl Outcome {
    /// Decide from the failures and the snapshot taken before the change.
    pub fn of(failed: Vec<String>, snapshot: Option<&str>) -> Outcome {
        if failed.is_empty() {
            return Outcome::Healthy;
        }
        match snapshot {
            Some(id) => Outcome::Revert {
                failed,
                snapshot: id.to_string(),
            },
            None => Outcome::FailedWithoutRevert { failed },
        }
    }
}

/// Whether to refuse before the change, and what to say.
///
/// The whole pre-flight decision, so it can be tested without a machine that happens to lack a
/// snapshot provider — the condition it exists for is the one hardest to reproduce on the
/// developer's own box, which is exactly why it must not live only in the wiring.
///
/// A dry run never refuses: it changes nothing, so there is nothing it might fail to revert.
pub fn refusal_if_unrevertable(
    checks: &[Check],
    has_provider: bool,
    dry_run: bool,
) -> Option<String> {
    if dry_run || has_provider || checks.is_empty() {
        return None;
    }
    Some(cannot_revert_refusal(checks))
}

/// The refusal issued *before* a change, when health checks are declared and nothing could
/// revert them.
///
/// 7f's exit condition in those words: it fails loudly and says it cannot revert, **before it
/// starts**. Finding this out afterwards is finding it out too late — the upgrade has already
/// happened and the check can only confirm the damage.
pub fn cannot_revert_refusal(checks: &[Check]) -> String {
    let mut named: Vec<&str> = checks.iter().map(|c| c.subject.as_str()).collect();
    named.sort_unstable();
    named.dedup();
    format!(
        "refusing to start: {} health check(s) are declared ({}) and this machine has no \
         snapshot provider, so a failing check could not revert anything.\n  \
         A health check that cannot revert reports the breakage and leaves it in place, which \
         is worse than not checking — you would have been told the machine is broken and given \
         no way back.\n  \
         Either set up a snapshot provider (btrfs, timeshift, or Windows System Restore), or \
         remove the health checks.",
        checks.len(),
        named.join(", ")
    )
}

/// What a revert says, in 7f's words.
pub fn reverted_message(failed: &[String], snapshot: &str) -> String {
    format!(
        "health check failed ({}) — restoring the snapshot taken before this change ({}).",
        failed.join(", "),
        snapshot
    )
}

/// What a failure with nothing to revert to says.
pub fn not_reverted_message(failed: &[String]) -> String {
    format!(
        "health check failed ({}) and there is no snapshot to restore — the machine is left as \
         the change made it.",
        failed.join(", ")
    )
}

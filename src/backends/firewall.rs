//! Driving whichever firewall this machine runs (Part XI).
//!
//! **Rows, not Rust** — K17's ruling, applied here because it was the reason 7o was scheduled
//! after it: `ufw`, `firewalld` and Windows Defender are three commands for one idea, and a
//! machine running a fourth should not wait for a Shall release. The shipped three are rows in
//! `firewall_adapters.toml`, parsed by the loader a user's own row goes through, because an
//! adapter mechanism the built-ins bypass is one nobody has tested.
//!
//! **One spelling across firewalls is the whole point (XI.2).** `firewall:22/tcp` means the
//! same thing on the Debian laptop and the Windows workstation — which a user-defined
//! `[[backend]]` naming `ufw` could never do, and which is why this is a built-in rather than
//! something the onboarder covers.

use crate::core::adapter::{self, AdapterRow, Detected};
use crate::model::firewall::{Direction, Proto, Rule};
use serde::Deserialize;

/// One firewall: how to tell it is here, and the commands that open, close, list and set the
/// default policy.
#[derive(Debug, Clone, Deserialize)]
pub struct FirewallAdapter {
    pub name: String,
    pub detect: String,
    #[serde(default)]
    pub os: Option<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    #[serde(default)]
    pub default_in: Vec<String>,
    #[serde(default)]
    pub default_out: Vec<String>,
    pub list: Vec<String>,
    /// A regex whose first two capture groups are the port and the protocol.
    pub list_pattern: String,
    /// A regex matching the line on which a firewall states its default policy; its first
    /// capture group is the body of that line. Absent is a firewall that cannot say — which
    /// `adopt` reports rather than guesses, because a default policy written from a guess is
    /// the one line that can make a machine unreachable.
    ///
    /// Where the policy is stated is per-firewall; how a pair is written is not, so the body is
    /// read with one shared pair grammar ([`DEFAULT_PAIR`]) rather than a regex per vendor.
    #[serde(default)]
    pub default_pattern: Option<String>,
}

/// `deny (incoming)` — the policy, then the direction it applies to in parentheses. The one
/// spelling every firewall in the shipped table that states a default uses, which is what makes
/// it a convention rather than a vendor quirk.
const DEFAULT_PAIR: &str = r"(\w+)\s*\((\w+)\)";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FirewallAdapterFile {
    #[serde(default)]
    pub firewall: Vec<FirewallAdapter>,
}

const BUILTIN: &str = include_str!("firewall_adapters.toml");

impl FirewallAdapter {
    fn fill(args: &[String], port: &str, proto: &str, policy: &str) -> Vec<String> {
        adapter::fill(
            args,
            &[("{port}", port), ("{proto}", proto), ("{policy}", policy)],
        )
    }

    /// The command that opens a port.
    pub fn allow_command(&self, port: u16, proto: Proto) -> Vec<String> {
        Self::fill(&self.allow, &port.to_string(), &proto.to_string(), "")
    }

    /// The command that closes one.
    pub fn deny_command(&self, port: u16, proto: Proto) -> Vec<String> {
        Self::fill(&self.deny, &port.to_string(), &proto.to_string(), "")
    }

    /// The command that sets a default policy, if this firewall can express one in that
    /// direction. `None` is a refusal the caller reports by name — never a silent skip.
    pub fn default_command(&self, direction: Direction, policy: &str) -> Option<Vec<String>> {
        let args = match direction {
            Direction::Incoming => &self.default_in,
            Direction::Outgoing => &self.default_out,
        };
        if args.is_empty() {
            return None;
        }
        Some(Self::fill(args, "", "", policy))
    }

    pub fn list_command(&self) -> Vec<String> {
        self.list.clone()
    }

    /// The rules this firewall reports as in force.
    ///
    /// A line that does not parse is skipped rather than guessed at: a firewall's status output
    /// carries headers, chains and comments, and inventing a rule out of one would put a
    /// phantom in the plan — the S22/S23 class, one domain over.
    pub fn parse_rules(&self, output: &str) -> Vec<Rule> {
        let Ok(re) = crate::utils::regex_cache::compiled(&self.list_pattern) else {
            return Vec::new();
        };
        // Every match on every line, not the first per line: `ufw status` puts one rule on a
        // line, while `firewall-cmd --list-ports` puts them all on one. A per-line parser reads
        // the second firewall as having a single rule and silently drops the rest, which would
        // make `sync` "correct" drift that was never there.
        let mut out = Vec::new();
        for line in output.lines() {
            for caps in re.captures_iter(line) {
                let (Some(port), Some(proto)) = (caps.get(1), caps.get(2)) else {
                    continue;
                };
                if let Ok(rule) = Rule::parse(&format!("{}/{}", port.as_str(), proto.as_str())) {
                    if !out.contains(&rule) {
                        out.push(rule);
                    }
                }
            }
        }
        out
    }

    /// The default policies this firewall reports as in force, for `adopt` to offer.
    ///
    /// A direction word this model has no `Direction` for is skipped rather than widened: ufw
    /// prints `disabled (routed)`, and reading that as an incoming or outgoing policy would
    /// write a line nobody measured. `None` from [`Self::default_command`] is the mirror
    /// refusal — a policy the firewall cannot set is not one to adopt either.
    pub fn parse_defaults(&self, output: &str) -> Vec<(Direction, String)> {
        let Some(pattern) = self.default_pattern.as_deref() else {
            return Vec::new();
        };
        let (Ok(line_of), Ok(pair)) = (
            crate::utils::regex_cache::compiled(pattern),
            crate::utils::regex_cache::compiled(DEFAULT_PAIR),
        ) else {
            return Vec::new();
        };
        let mut out: Vec<(Direction, String)> = Vec::new();
        for line in line_of.captures_iter(output) {
            let Some(body) = line.get(1) else { continue };
            for caps in pair.captures_iter(body.as_str()) {
                let (Some(policy), Some(direction)) = (caps.get(1), caps.get(2)) else {
                    continue;
                };
                let direction = match direction.as_str().to_ascii_lowercase().as_str() {
                    "incoming" | "inbound" | "in" => Direction::Incoming,
                    "outgoing" | "outbound" | "out" => Direction::Outgoing,
                    _ => continue,
                };
                let policy = policy.as_str().to_ascii_lowercase();
                if self.default_command(direction, &policy).is_none() {
                    continue;
                }
                if !out.iter().any(|(d, _)| *d == direction) {
                    out.push((direction, policy));
                }
            }
        }
        out
    }
}

impl AdapterRow for FirewallAdapter {
    const WHAT: &'static str = "firewall adapter";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A firewall Shall will drive. It must be able to open a port, close one, and say what is
    /// in force — a firewall it cannot read is one whose drift could never be seen, which is a
    /// `firewall:` line that reports converged over a perimeter nobody looked at.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.detect.trim().is_empty() {
            return Some("it has no `detect` command");
        }
        if self.allow.is_empty() || self.deny.is_empty() {
            return Some("it cannot both open and close a port");
        }
        if self.list.is_empty() {
            return Some("it cannot list its rules, so drift could never be seen");
        }
        None
    }
}

impl Detected for FirewallAdapter {
    fn detect_command(&self) -> &str {
        &self.detect
    }
}

/// The `[[firewall]]` rows the repo carries, through the same approval every adapter file goes
/// through (7a/II.12).
///
/// Here rather than in either caller because there are two — the phase that applies a perimeter
/// and the `adopt` that offers one — and a second reader would be a second answer to "which
/// rows does this repo have".
pub fn user_rows(config: &crate::config::Config) -> Vec<FirewallAdapter> {
    let layout = config.layout();
    let path = layout.adapter_firewall_file();
    let Some(body) =
        crate::backends::onboarder::read_approved_definitions(&path, &layout.locks_dir())
    else {
        return Vec::new();
    };
    match toml::from_str::<FirewallAdapterFile>(&body) {
        Ok(f) => f.firewall,
        Err(e) => {
            tracing::warn!(
                "{}",
                crate::app::adapters::cannot_use(
                    crate::app::adapters::surface("firewall").expect("a declared surface"),
                    e,
                )
            );
            Vec::new()
        }
    }
}

/// Every firewall adapter this machine knows: the shipped rows, then the user's.
pub fn adapters(user_rows: Vec<FirewallAdapter>) -> Vec<FirewallAdapter> {
    let shipped: FirewallAdapterFile =
        toml::from_str(BUILTIN).expect("the shipped firewall_adapters.toml must parse");
    adapter::merge(shipped.firewall.into_iter().chain(user_rows))
}

/// The adapter for the firewall this machine is running.
pub fn detect<'a>(
    adapters: &'a [FirewallAdapter],
    present: &dyn Fn(&str) -> bool,
) -> Option<&'a FirewallAdapter> {
    // No firewall row names a liveness file yet, so this is the identity of the old behaviour.
    // Named rather than left to a default: `firewall-cmd` present with firewalld STOPPED is the
    // same shape as the systemd finding, and the next person to measure it has the seam here.
    adapter::first_present(adapters, present, &|p| std::path::Path::new(p).exists())
}

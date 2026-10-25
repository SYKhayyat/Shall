//! `firewall:22/tcp` — a perimeter you declare (Part XI, N1–N7).
//!
//! **The lockout check is this feature's precondition, not one of its features.** Everything
//! else here manages rules; this decides whether Shall is allowed to close the port carrying
//! the connection it is being typed over. Building the backend before the check is building
//! the lockout — so the check lives at the bottom of the module and everything above it is
//! written knowing it exists.
//!
//! Rulings of 2026-07-24:
//!
//! - **N4** — the default policy may be a `firewall:` statement *or* a `preferences.toml` key,
//!   and where both speak, **the declaration wins**: it is the one you can read, review and
//!   share, and a machine-local key silently overriding it would be the invisible answer
//!   beating the visible one.
//! - **N5** — removing a rule restores the firewall's own default, exactly as a removed
//!   `setting:` resets to the schema default. Restoring a per-rule prior state would mean
//!   keeping a per-rule store of it.
//! - **N6** — declaring rules *and* linking a ruleset file **warns and applies both**, with the
//!   `firewall:` line winning where they disagree. A base file plus overrides is legible; two
//!   silent owners are not.
//! - **N7** — unattended `watch` **reverts** drift, because drift is corrected everywhere else
//!   in this model — *except* where the revert would close the session's own port, which is
//!   reported and left alone. An un-reverted rule is fixed tomorrow; a reverted one can be a
//!   machine you cannot reach.

use std::fmt;

/// What a `firewall:` line names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// `firewall:22/tcp` — a port, and the protocol it is spoken over.
    Port { port: u16, proto: Proto },
    /// `firewall:default/incoming` — the policy for traffic no rule mentions. The most
    /// consequential line in a firewall, which is why it is declarable at all (N4).
    Default { direction: Direction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proto {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

impl fmt::Display for Proto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Proto::Tcp => "tcp",
            Proto::Udp => "udp",
        })
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Direction::Incoming => "incoming",
            Direction::Outgoing => "outgoing",
        })
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::Port { port, proto } => write!(f, "{}/{}", port, proto),
            Rule::Default { direction } => write!(f, "default/{}", direction),
        }
    }
}

impl Rule {
    /// Parse the name half of a `firewall:` line. One parser, so the refusal and the adapter
    /// cannot disagree about what a rule is.
    pub fn parse(name: &str) -> Result<Rule, String> {
        let (head, tail) = name.trim().split_once('/').ok_or_else(|| {
            format!(
                "`{}` is not a firewall rule. Write a port and protocol (`22/tcp`) or a \
                 default policy (`default/incoming`).",
                name
            )
        })?;
        let (head, tail) = (head.trim(), tail.trim());

        if head.eq_ignore_ascii_case("default") {
            return match tail.to_lowercase().as_str() {
                "incoming" => Ok(Rule::Default {
                    direction: Direction::Incoming,
                }),
                "outgoing" => Ok(Rule::Default {
                    direction: Direction::Outgoing,
                }),
                _ => Err(format!(
                    "`default/{}` names no direction. It is `default/incoming` or \
                     `default/outgoing`.",
                    tail
                )),
            };
        }

        let port: u16 = head
            .parse()
            .map_err(|_| format!("`{}` is not a port number (1–65535).", head))?;
        if port == 0 {
            return Err("port 0 is not a port you can open.".to_string());
        }
        let proto = match tail.to_lowercase().as_str() {
            "tcp" => Proto::Tcp,
            "udp" => Proto::Udp,
            _ => {
                return Err(format!(
                    "`{}` is not a protocol. Shall speaks `tcp` and `udp`; anything else is a \
                     rule your firewall's own tool should write.",
                    tail
                ))
            }
        };
        Ok(Rule::Port { port, proto })
    }

    /// The port this rule concerns, if it concerns one. A default policy concerns them all,
    /// which is why the lockout check treats it separately.
    pub fn port(&self) -> Option<u16> {
        match self {
            Rule::Port { port, .. } => Some(*port),
            Rule::Default { .. } => None,
        }
    }
}

/// The local port carrying the connection Shall is being run over, if any.
///
/// Read from `SSH_CONNECTION` — the shell sets it to
/// `<client ip> <client port> <server ip> <server port>`, and the last field is the port this
/// session arrived on. A console session has no such variable and nothing to lose.
///
/// **Here rather than beside one caller**, because there are two perimeters now: the adapter
/// path that runs `ufw`, and the NixOS path that writes `networking.firewall.allowedTCPPorts`.
/// A lockout check that only one of them could ask is a lockout check on one host class.
pub fn session_port() -> Option<u16> {
    let raw = std::env::var("SSH_CONNECTION").ok()?;
    raw.split_whitespace().nth(3)?.parse().ok()
}

/// Whether applying `plan` would cut the connection Shall is being run over (Part XI's
/// precondition).
///
/// `session_port` is the local port carrying the controlling connection — `None` when Shall is
/// on a console and there is nothing to lose.
///
/// **A tightened default policy counts.** Closing `default/incoming` is not a rule about port
/// 22, but it closes port 22 all the same unless something else opens it — and the whole point
/// of this check is that the user is on the far end of that port right now.
pub fn would_close_session(
    removing: &[Rule],
    default_incoming_becomes_deny: bool,
    still_open: &[Rule],
    session_port: Option<u16>,
) -> Option<u16> {
    let port = session_port?;

    // Explicitly taking away the rule that keeps this connection alive.
    if removing.iter().any(|r| r.port() == Some(port)) {
        return Some(port);
    }

    // Or shutting the door everything unlisted came through, with nothing left to hold it open.
    if default_incoming_becomes_deny && !still_open.iter().any(|r| r.port() == Some(port)) {
        return Some(port);
    }
    None
}

/// The refusal, in the words a locked-out user would have needed.
pub fn lockout_refusal(port: u16, scope: crate::app::sync::guard::GuardScope) -> String {
    format!(
        "refusing to apply the firewall change: it would close port {}, which is carrying this \
         session.\n  \
         Shall is being run over that port, so applying this would end the connection and \
         leave no way back in.\n  \
         Declare `firewall:{}/tcp` to keep it open, or make this change from the machine's own \
         console.\n  \
         (refused during {})",
        port,
        port,
        scope.during()
    )
}

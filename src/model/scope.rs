//! `@scope=user` / `@scope=system` — is Shall acting for you, or for the machine? (U19)
//!
//! Ruled 2026-07-24. Shall used to act, implicitly, as whoever typed the command. The Linux
//! backends mostly agree with that by accident; the Windows registry cannot, because `HKCU`
//! and `HKLM` are a choice with no default that is right for both. So the question is asked on
//! the three statements where it can vary — `setting:`, `link:`, `shim:` — and **defaults to
//! whatever the underlying store does anyway**, so a user writes `@scope=` only to override.
//!
//! **Writing the default is not an error** (owner, 2026-07-24). `@scope=user` on a store whose
//! default is already user is accepted and means exactly what it says. A configuration is
//! allowed to state a thing it also gets for free: saying it out loud is how a reader learns
//! the answer without going and looking it up, and refusing it would punish the person being
//! explicit.

use std::fmt;

/// Who a declaration is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// This account only — `HKCU`, `gsettings`, `~/.local/bin`.
    User,
    /// Every account on the machine — `HKLM`, `/etc`, `/usr/local/bin`.
    System,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::User => "user",
            Scope::System => "system",
        }
    }

    /// The legal spellings, for an error message that lists them rather than restating them.
    pub fn vocabulary() -> String {
        format!("{}, {}", Scope::User.as_str(), Scope::System.as_str())
    }

    pub fn parse(value: &str) -> Option<Scope> {
        match value.trim() {
            "user" => Some(Scope::User),
            "system" => Some(Scope::System),
            _ => None,
        }
    }

    /// What a declaration means, given what the user wrote and what this store does by
    /// default. `None` written is the default — which is the whole point of having one.
    pub fn resolve(written: Option<&str>, default: Scope) -> Scope {
        written.and_then(Scope::parse).unwrap_or(default)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

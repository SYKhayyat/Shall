//! Hardware-token secrets: what a missing token looks like, and what `watch` does with one
//! (T3, T4).
//!
//! An `@decrypt` line whose identity is a hardware token — an age plugin like `age-plugin-yubikey`
//! — needs a physical touch to decrypt. That is fine at a terminal and a trap everywhere else: a
//! `sync` in a script, or a `watch` tick at 3am, would sit forever on a prompt nobody will
//! answer.
//!
//! Two rules:
//! - **T3:** a decrypt that does not complete times out, and Shall names the token and the
//!   identity file rather than passing the plugin's own prompt text through.
//! - **T4:** an unattended `watch` tick does not even try a touch-required line — it skips it and
//!   says so once, rather than blocking the whole reconcile.
//!
//! Pure: recognising a plugin identity, and the two messages. Reading the file and enforcing the
//! timeout are the caller's.

use crate::core::adapter::{self, AdapterRow};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// A declared secret-decryption provider (U38): sops, Vault, 1Password, a KMS, plain GPG — any
/// command that turns a reference into plaintext. `age` and `sops` stay built in (age carries the
/// hardware-token handling above); this opens the door to the rest as data, the same "rows, not
/// Rust" move the other provider surfaces made.
///
/// **This is the one surface where openness is not cheap, and its rule is the strictest.** A
/// decrypt provider's output *is* a secret, so a provider is bound by the T-series plaintext
/// rules: the plaintext must come out on **stdout** and nowhere else — never a file it writes,
/// never a line it logs — so Shall captures it in memory and restricts the destination before it
/// is written (T5), never backs it up (T1), and never lets it reach the repo (T2). A provider
/// that cannot promise stdout-only is **refused, not trusted** — and the promise is explicit:
/// `stdout_only = true` must be in the block, or the provider does not load. The unsafe reading
/// is never the default (V.80).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SecretProvider {
    pub name: String,
    /// Restrict to one OS (`std::env::consts::OS`). Absent means any.
    ///
    /// Every other adapter table has carried this since it was written; this one did not, so
    /// the single table whose rows hand a command a plaintext secret was the only one that
    /// could not be confined to the platform it was written for. A row that names no OS still
    /// applies everywhere, so no existing `adapters/secret.toml` changes meaning.
    #[serde(default)]
    pub os: Option<String>,
    /// The argv that decrypts. `{ref}` is the reference from the line (the source path or a
    /// secret id); `{identity}` is the `@identity=` value if one was given. Plaintext on stdout.
    pub decrypt: Vec<String>,
    /// The T-series promise, and it is required to be `true`: the provider writes the plaintext
    /// to stdout only. A block that omits it (default `false`) is refused — Shall will not hand a
    /// secret to a command that has not promised to keep it off disk and out of the logs.
    #[serde(default)]
    pub stdout_only: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecretProviderFile {
    #[serde(default)]
    pub secret: Vec<SecretProvider>,
}

impl SecretProvider {
    /// The `(program, args)` to run, with `{ref}` and `{identity}` filled in. Returns `None` when
    /// the block has no program (already rejected by `unusable`, but never unwrapped).
    pub fn command(
        &self,
        reference: &str,
        identity: Option<&str>,
    ) -> Option<(String, Vec<String>)> {
        adapter::program_and_args(adapter::fill(
            &self.decrypt,
            &[("{ref}", reference), ("{identity}", identity.unwrap_or(""))],
        ))
    }
}

impl AdapterRow for SecretProvider {
    const WHAT: &'static str = "secret provider";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A provider Shall will trust with a secret, or why it will not.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.decrypt.iter().all(|a| a.trim().is_empty()) {
            return Some("it has no `decrypt` command");
        }
        if !self.stdout_only {
            return Some(
                "it does not declare `stdout_only = true` — a secret provider must promise the \
                 plaintext reaches stdout only (never a file, never a log), or Shall will not \
                 hand it a secret (T-series)",
            );
        }
        None
    }
}

/// Every secret provider this machine can use, from `adapters/secret.toml`.
///
/// Rows for another platform are dropped here rather than failed on later: a `[[secret]]`
/// naming a Windows credential tool is not a provider a Linux box should pick and then fail
/// to run. A second row claiming a name is refused the way every other adapter table refuses
/// one — this table used to keep it silently, so which of two providers answered for `vault`
/// depended on file order and nothing said so.
pub fn providers(rows: Vec<SecretProvider>) -> Vec<SecretProvider> {
    adapter::merge(rows.into_iter().filter(|p| p.applies_here()))
}

/// How long a decrypt may run before Shall concludes it is waiting on a prompt nobody will
/// answer (T3). Generous — a real hardware touch is a few seconds, and a slow disk read of a
/// software key is well under this — so the timeout only fires on a genuine hang.
pub const DECRYPT_TIMEOUT: Duration = Duration::from_secs(30);

/// The age plugin an identity file names, if it is a hardware/interactive one.
///
/// An age plugin identity is `AGE-PLUGIN-<NAME>-1…`; the plugin's binary is `age-plugin-<name>`
/// and the touch is its doing. A software identity (`AGE-SECRET-KEY-1…`) has no plugin and no
/// touch, so it returns `None` and is decrypted the ordinary way. Case-insensitive, because an
/// identity file is user-written.
pub fn plugin_of(identity_contents: &str) -> Option<String> {
    for line in identity_contents.lines() {
        let line = line.trim();
        // The marker can be the identity itself or a `#` comment age-keygen writes above it.
        let body = line.trim_start_matches('#').trim();
        let upper = body.to_ascii_uppercase();
        if let Some(rest) = upper.strip_prefix("AGE-PLUGIN-") {
            // `AGE-PLUGIN-YUBIKEY-1ABC…` → `yubikey`.
            let name: String = rest.chars().take_while(|c| *c != '-').collect();
            if !name.is_empty() {
                return Some(name.to_ascii_lowercase());
            }
        }
    }
    None
}

/// T3: what a timed-out decrypt says. Names the token and the identity file, not the plugin's
/// own text — a message Shall owns and can be acted on.
pub fn token_timeout_message(source: &Path, identity: &Path, plugin: Option<&str>) -> String {
    let token = match plugin {
        Some(p) => format!("the `{}` hardware token", p),
        None => "the hardware token".to_string(),
    };
    format!(
        "decrypting {} timed out after {}s — {} did not respond.\n  \
         Its identity is {}. If this is a key that needs a physical touch, do it at a terminal, \
         or run this where the token is present.",
        source.display(),
        DECRYPT_TIMEOUT.as_secs(),
        token,
        identity.display()
    )
}

/// T4: what an unattended `watch` says when it skips a touch-required line. Said once.
pub fn watch_skip_message(source: &Path, plugin: &str) -> String {
    format!(
        "skipping the encrypted {} this tick — its `{}` identity needs a physical touch, and \
         this is an unattended `watch` run. It will apply on the next `sync` you run yourself.",
        source.display(),
        plugin
    )
}

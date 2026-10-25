//! What a download-shaped backend actually resolved to: `locks/<backend>.toml` (VIII.2, D6).
//!
//! A version is not the identity of a downloaded artifact. `github:sharkdp/fd@version=10.2.0`
//! names one release and that release ships a `.deb`, an `.rpm`, a `.tar.gz` and a bare
//! binary — so a lock recording only the version leaves the artifact free to change under a
//! pinned declaration, which is the bug Part VIII exists to close.
//!
//! The hash is here rather than in the declaration because one hash cannot cover an asset that
//! varies by machine (D6): a shared module says `github:x/y` and the Ubuntu box downloads the
//! `.deb` while the Fedora box downloads the `.rpm`. A per-machine record can describe both; a
//! hand-written `@sha256=` cannot describe either without pinning the format first.
//!
//! **A recorded hash is a record, not a policy.** It says what was downloaded, so a change is
//! visible in `shall diff` and a re-download that differs is an error. It does not demand that
//! the user pre-declare anything.

use crate::core::ledger::LockFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One resolved artifact. Every field is generated — nothing here is typed by a user.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ArtifactLock {
    /// The release this came from, as the backend names it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The asset filename that was chosen, so a re-resolve that picks differently is visible.
    pub asset: String,
    /// Where it came from. Recorded because the asset name alone does not identify a file.
    pub url: String,
    /// The `formats` entry that matched, as VIII.2 spells it.
    pub format: String,
    /// Which rule chose this format — the line's `@asset=`/`@formats=`, or the built-in default
    /// (D14). Kept so `why` can answer "why this file and not the `.deb` I expected" without
    /// re-running a network selection. `serde(default)` so a lock written before D14 still
    /// parses; it simply has no reason to show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_by: Option<String>,
    /// The hash of the bytes that were installed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The system installer that owns this artifact once it is placed (D5) — `dpkg` for a
    /// `.deb`, `rpm` for an `.rpm`. `None` is the ordinary case: Shall unpacked it and put it on
    /// PATH itself. When set, removal, upgrade and dedup route through this manager rather than
    /// through a file delete, and `system_package` is the name it was recorded under.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_by: Option<String>,
    /// The package name the system manager knows this artifact as (read from the file at install
    /// time). Only meaningful alongside `installed_by`; it is what `dpkg -r`/`rpm -e` and the
    /// unmanaged-dedup key on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_package: Option<String>,
}

/// `locks/<backend>.toml`, keyed by the package name the declaration used. A `BTreeMap` so the
/// file is ordered and diffs cleanly in git.
///
/// A declaration locks a *list* because `@asset=all` installs every match (VIII.2), and one
/// entry per declaration could only describe the first of them.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ArtifactLedger {
    #[serde(default, flatten)]
    entries: BTreeMap<String, Vec<ArtifactLock>>,
}

impl LockFile for ArtifactLedger {
    const WHAT: &'static str = "the artifact ledger";
}

impl ArtifactLedger {
    /// What a declaration resolved to, in selection order. Empty when nothing is locked.
    pub fn locked(&self, name: &str) -> &[ArtifactLock] {
        self.entries.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Record everything one declaration installed. The whole set is replaced, because a
    /// declaration that now resolves to fewer artifacts must not keep the ones it dropped.
    pub fn record(&mut self, name: impl Into<String>, locks: Vec<ArtifactLock>) {
        self.entries.insert(name.into(), locks);
    }

    /// Drop a package's entry. The lock describes what is installed, so a removal that left
    /// the entry behind would pin a future install to an artifact chosen for a different
    /// declaration.
    pub fn forget(&mut self, name: &str) -> bool {
        self.entries.remove(name).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &[ArtifactLock])> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    /// Every system package these declarations installed through a system manager (D5), as
    /// `(installer, package_name)` pairs. The unmanaged crawl and `check` subtract these so a
    /// `.deb` a `github:` line handed to `dpkg` is not also reported as apt-visible drift, and
    /// `purge-undeclared` defers to the recorded installer rather than deleting it.
    pub fn system_packages(&self) -> Vec<(String, String)> {
        self.entries
            .values()
            .flatten()
            .filter_map(|l| match (&l.installed_by, &l.system_package) {
                (Some(installer), Some(pkg)) => Some((installer.clone(), pkg.clone())),
                _ => None,
            })
            .collect()
    }
}

/// What a re-download of a whole declaration must satisfy, when `@asset=all` locked several.
///
/// The set is compared by name, not by position: a release that reorders its assets is not a
/// change to what was installed. Anything else — a name that was not locked, or one that is
/// locked and no longer resolved — is the same objection [`verify_against`] raises for one.
pub fn verify_set(
    locked: &[ArtifactLock],
    resolved: &[(String, Option<String>)],
) -> Option<String> {
    for (asset, sha) in resolved {
        let Some(entry) = locked.iter().find(|l| &l.asset == asset) else {
            return Some(format!(
                "the lock records {} and this resolved to `{}` as well. Run `shall lock` if \
                 the change is intended.",
                named(locked),
                asset
            ));
        };
        if let Some(objection) = verify_against(entry, asset, sha.as_deref()) {
            return Some(objection);
        }
    }
    if let Some(dropped) = locked
        .iter()
        .find(|l| !resolved.iter().any(|(a, _)| a == &l.asset))
    {
        return Some(format!(
            "the lock records `{}` and this release no longer offers it. Run `shall lock` if \
             the change is intended.",
            dropped.asset
        ));
    }
    None
}

fn named(locks: &[ArtifactLock]) -> String {
    locks
        .iter()
        .map(|l| format!("`{}`", l.asset))
        .collect::<Vec<_>>()
        .join(", ")
}

/// What a re-download must satisfy to be the artifact the lock describes.
///
/// Returns the objection, or `None` when the download is what was locked. A mismatch is an
/// error rather than a re-selection: selecting a different asset because the pinned one failed
/// its hash would turn a supply-chain alarm into a silent substitution (VIII.2).
pub fn verify_against(lock: &ArtifactLock, asset: &str, sha256: Option<&str>) -> Option<String> {
    if lock.asset != asset {
        return Some(format!(
            "the lock records `{}` and this resolved to `{}`. Run `shall lock` if the change \
             is intended.",
            lock.asset, asset
        ));
    }
    match (&lock.sha256, sha256) {
        (Some(locked), Some(got)) if !locked.eq_ignore_ascii_case(got) => Some(format!(
            "`{}` does not match the hash in the lock.\n  locked: {}\n  got:    {}",
            asset, locked, got
        )),
        // A hash was recorded and the new download carries NONE: VIII.2 exists to stop a pass
        // going through unobjected, and skipping the check here is how a re-download whose
        // hashing failed (or a caller that forgot to hash) would substitute bytes silently.
        (Some(locked), None) => Some(format!(
            "`{}` came back with no checksum to compare against the lock's `{}`. Hashing \
             failed or was skipped; refusing rather than accepting unverified bytes.",
            asset, locked
        )),
        _ => None,
    }
}

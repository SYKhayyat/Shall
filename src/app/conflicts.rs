// src/app/conflicts.rs
//
// Cross-backend conflict detection. Each backend resolves its own dependencies in isolation,
// so nothing otherwise notices when two ecosystems are told to install the SAME logical tool
// at incompatible versions (apt:nodejs@18 vs nix:nodejs@20), or when two managers would both
// provide it (a shadowing risk on PATH). This runs at plan time over the resolved desired set
// and reports those cross-ecosystem clashes — something single-backend planners can't see.
//
// The analysis is pure (specs in, findings out) and unit-tested. It deliberately errs toward
// exact-name matching (plus a small table of well-known equivalents) so it does not cry wolf.

use crate::core::PackageSpec;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ConflictKind {
    /// The same tool is pinned to different versions by different backends.
    VersionMismatch,
    /// The same tool would be installed by more than one backend (PATH shadowing risk).
    MultipleProviders,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Conflict {
    /// The canonical package name the clash is about.
    pub name: String,
    pub kind: ConflictKind,
    /// Each backend that provides it, with the pinned version if any.
    pub providers: Vec<(String, Option<String>)>,
}

/// Map a raw package name to a canonical form so well-known equivalents across ecosystems
/// (nodejs/node, python3/python, golang/go) are recognized as the same tool. Everything else
/// is compared case-insensitively by its own name.
pub fn canonical_name(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "nodejs" | "node" | "nodejs-lts" => "node".to_string(),
        "python3" | "python" | "python-3" => "python".to_string(),
        "golang" | "go" => "go".to_string(),
        "postgresql" | "postgres" => "postgresql".to_string(),
        "docker" | "docker-ce" | "docker.io" => "docker".to_string(),
        other => other.to_string(),
    }
}

/// The version a spec pins, if any (`@version=…`).
fn pinned_version(spec: &PackageSpec) -> Option<String> {
    spec.options.one("version").map(str::to_string)
}

/// Detect cross-backend conflicts in a flat set of desired specs. A group of specs sharing a
/// canonical name and drawn from more than one distinct backend is a conflict:
/// `VersionMismatch` when two of them pin different versions, else `MultipleProviders`.
pub fn detect_conflicts(specs: &[PackageSpec]) -> Vec<Conflict> {
    // canonical name -> (backend -> pinned version). BTreeMap keeps output deterministic.
    let mut groups: BTreeMap<String, BTreeMap<String, Option<String>>> = BTreeMap::new();
    for spec in specs {
        let canon = canonical_name(&spec.name);
        let entry = groups.entry(canon).or_default();
        // If the same backend lists it twice, prefer a pinned version over none.
        let slot = entry.entry(spec.backend.clone()).or_insert(None);
        if slot.is_none() {
            *slot = pinned_version(spec);
        }
    }

    let mut conflicts = Vec::new();
    for (name, by_backend) in groups {
        if by_backend.len() < 2 {
            continue; // only one backend provides it — no cross-backend clash
        }
        let providers: Vec<(String, Option<String>)> = by_backend.into_iter().collect();

        // Distinct pinned versions among providers?
        let mut versions: Vec<&String> = providers.iter().filter_map(|(_, v)| v.as_ref()).collect();
        versions.sort();
        versions.dedup();
        let kind = if versions.len() > 1 {
            ConflictKind::VersionMismatch
        } else {
            ConflictKind::MultipleProviders
        };
        conflicts.push(Conflict {
            name,
            kind,
            providers,
        });
    }
    conflicts
}

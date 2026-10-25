//! `dotfiles:./tree` — a folder of files, linked where they belong (XIII.21, U22–U25).
//!
//! A dotfiles repo is already a tree that mirrors `$HOME`. Writing forty `link:` lines to say
//! so is forty chances to forget one. This statement says it once.
//!
//! **Per file, never per directory (U22).** One symlink at `~/.config/nvim` takes the whole
//! directory hostage: every cache, session file and plugin lockfile the application later
//! writes lands inside the git-tracked config repo, and `bundle` then hands it to whoever the
//! backup goes to. Linking each file leaves the directory the user's and puts nothing in the
//! repo that was not put there deliberately. The cost is walking the tree each sync and one
//! ledger row per file, and that is the cost worth paying.
//!
//! The rows are written by `Dotfiles::links`, which expands a tree into the `link:` lines it
//! stands for — that expansion is what makes the teardown, the `<dest>.shall-backup` and the
//! removal guard the tree's as much as a hand-written line's.
//!
//! **The tree never decrypts (U24).** A `.age` file in it is copied as the ciphertext it is.
//! Deciding by file extension is magic, and magic that silently writes plaintext; secrets stay
//! on explicit `link:` lines where `@decrypt=` is written down.
//!
//! This module is pure: it walks a tree and computes destinations and collisions. Nothing here
//! touches a destination — that is the caller's, after the plan has been shown.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One file the tree would place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// The file inside the tree.
    pub source: PathBuf,
    /// Where it belongs, under the target root.
    pub destination: PathBuf,
    /// Its path relative to the tree root — what makes the mapping legible in a plan.
    pub relative: PathBuf,
}

/// What a tree would do, and what stands in its way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreePlan {
    /// Every file, in a stable order so a plan reads the same twice.
    pub placements: Vec<Placement>,
    /// Destinations that already hold something Shall did not put there (U23). The run is
    /// refused until these have been seen.
    pub collisions: Vec<PathBuf>,
}

impl TreePlan {
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }
}

/// Which files a tree skips. Not configurable, and deliberately short: a dotfiles tree is a
/// tree of dotfiles, and the things below are never one of them.
///
/// `.git` is the big one — mirroring it into `$HOME` would put a second repository there.
fn is_skipped(name: &str) -> bool {
    matches!(name, ".git" | ".gitignore" | ".gitattributes" | ".DS_Store")
}

/// Walk `tree`, mapping every file to its place under `target_root`.
///
/// `owned` answers "did Shall put this here?" for a destination that already exists — injected
/// rather than read here so this stays pure and the ownership rule lives in one place.
pub fn plan(
    tree: &Path,
    target_root: &Path,
    exists: &dyn Fn(&Path) -> bool,
    owned: &dyn Fn(&Path) -> bool,
) -> std::io::Result<TreePlan> {
    let mut out = TreePlan::default();
    let mut files = Vec::new();
    walk(tree, tree, &mut files)?;
    // Sorted so the plan — and therefore the confirmation a user reads — is stable.
    files.sort();

    for relative in files {
        let source = tree.join(&relative);
        let destination = target_root.join(&relative);
        if exists(&destination) && !owned(&destination) {
            out.collisions.push(destination.clone());
        }
        out.placements.push(Placement {
            source,
            destination,
            relative,
        });
    }
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if is_skipped(&name) {
            continue;
        }
        let path = entry.path();
        // `symlink_metadata`: a symlink inside the tree is a file to place, not a directory to
        // descend into — following it could walk out of the tree entirely.
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            walk(root, &path, out)?;
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// Two trees that would place a file at the same destination (U25).
///
/// An error naming both, rather than a rule forbidding a second tree: the statement takes a
/// path, so several are natural, and the thing that actually goes wrong is two of them
/// claiming one destination. That is II.7 rule 5 reached by a new road, not a new rule.
pub fn conflicting_destinations(plans: &[(String, TreePlan)]) -> BTreeMap<PathBuf, Vec<String>> {
    let mut by_destination: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for (tree, plan) in plans {
        for p in &plan.placements {
            by_destination
                .entry(p.destination.clone())
                .or_default()
                .push(tree.clone());
        }
    }
    by_destination.retain(|_, trees| {
        trees.sort();
        trees.dedup();
        trees.len() > 1
    });
    by_destination
}

/// The destinations a set of plans would own, for the teardown ledger.
pub fn destinations(plan: &TreePlan) -> BTreeSet<PathBuf> {
    plan.placements
        .iter()
        .map(|p| p.destination.clone())
        .collect()
}

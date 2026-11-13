//! Finding the dotfiles a person already keeps in a repository (#59).
//!
//! `adopt` could report packages and nothing else about `$HOME`, so a machine that had been
//! managed for years with `stow` or a hand-rolled script looked empty to it — every one of those
//! `link:` lines had to be written by hand from memory.
//!
//! **What counts as evidence is a symlink whose target is inside a git working tree**, and
//! nothing else. That is narrower than "a config file exists", which is what every application
//! writes on its own and not a decision anybody made, and wider than "a `stow` layout", which
//! would only find the four tools that name themselves. A link into version control is a person
//! saying *this file lives there and is mine* — the one fact in `$HOME` that is unambiguously
//! about intent (`V.249`).
//!
//! A broken link is dropped rather than declared: its target is not on disk, so the line would
//! describe a file nobody has, and `sync` would refuse it by name on the next run.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A `$HOME` path that is a symlink into version control.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManagedLink {
    /// Where the real bytes live — the repository file, already canonical.
    pub source: PathBuf,
    /// The link itself, as it sits in `$HOME`.
    pub target: PathBuf,
}

/// How deep below a root the scan looks. `~/.config/nvim/init.lua` is a dotfile somebody links;
/// `~/.config/nvim/plugin/bundle/x/y.vim` is a plugin manager's cache, and its links are the
/// plugin manager's business, not a declaration anyone wants to read.
const DEPTH: usize = 2;

/// A ceiling, so one pathological directory cannot turn `adopt` into a hang. Reaching it is
/// reported rather than silent.
pub const MAX_LINKS: usize = 500;

/// Directory names that are state or cache by convention. A link found under one of these is a
/// tool managing itself.
const NOT_CONFIG: &[&str] = &[
    ".cache",
    "Cache",
    "CacheStorage",
    "Code Cache",
    "GPUCache",
    "Trash",
    "node_modules",
];

/// The roots the scan starts from, in the order they are printed.
pub fn roots(home: &Path) -> Vec<PathBuf> {
    ["", ".config", ".local/share", ".ssh"]
        .iter()
        .map(|rel| {
            if rel.is_empty() {
                home.to_path_buf()
            } else {
                home.join(rel)
            }
        })
        .filter(|p| p.is_dir())
        .collect()
}

/// Every managed link under `home`, sorted by where it sits.
pub fn scan(home: &Path) -> (Vec<ManagedLink>, bool) {
    let mut out = Vec::new();
    let mut memo = HashMap::new();
    for root in roots(home) {
        // `$HOME` itself is walked one level only: `.bashrc` and `.gitconfig` are dotfiles, and
        // `~/Documents/2024/notes` is not something to crawl.
        let depth = if root == home { 1 } else { DEPTH };
        walk(&root, depth, home, &mut memo, &mut out);
        if out.len() >= MAX_LINKS {
            out.truncate(MAX_LINKS);
            out.sort();
            return (out, true);
        }
    }
    out.sort();
    (out, false)
}

/// Whether this path is inside a git working tree.
///
/// Walks up to the filesystem root: a dotfiles repository is wherever its owner put it, and
/// `~/projects/dotfiles/vim/vimrc` is as ordinary as `~/.dotfiles/vimrc`. `.git` is a directory
/// in a plain clone and a *file* in a worktree or submodule, so both count.
fn in_a_repository(path: &Path, memo: &mut HashMap<PathBuf, bool>) -> bool {
    let mut answer = false;
    let mut chain = Vec::new();
    for dir in path.ancestors() {
        if let Some(known) = memo.get(dir) {
            answer = *known;
            break;
        }
        chain.push(dir.to_path_buf());
        if dir.join(".git").exists() {
            answer = true;
            break;
        }
    }
    // Every directory on the way up shares the answer: they either all sit below the same
    // `.git` or all of them looked at the root and found none.
    for dir in chain {
        memo.insert(dir, answer);
    }
    answer
}

fn walk(
    dir: &Path,
    depth: usize,
    home: &Path,
    memo: &mut HashMap<PathBuf, bool>,
    out: &mut Vec<ManagedLink>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(std::result::Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        // `symlink_metadata`, so a link is seen as a link and not as whatever it points at.
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            if let Some(link) = as_managed_link(&path, home, memo) {
                out.push(link);
            }
            // Never descended into: a link to `$HOME` or to a repository root would walk the
            // same tree again, and the loop-guard would be a depth counter that does not exist.
            continue;
        }
        if !meta.is_dir() || depth <= 1 || path == home {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if NOT_CONFIG.contains(&name) {
                continue;
            }
        }
        walk(&path, depth - 1, home, memo, out);
    }
}

fn as_managed_link(
    path: &Path,
    home: &Path,
    memo: &mut HashMap<PathBuf, bool>,
) -> Option<ManagedLink> {
    // A relative target is relative to the link's own directory, which is how `readlink` and the
    // kernel read it — not to the working directory of the scan.
    let raw = std::fs::read_link(path).ok()?;
    let resolved = if raw.is_absolute() {
        raw
    } else {
        path.parent()?.join(raw)
    };
    // Canonical, and so also "must exist": a dangling link has no file to declare.
    let source = std::fs::canonicalize(&resolved).ok()?;
    // A link that stays inside `$HOME` is a rearrangement, not a dotfile kept in a repository —
    // and following it would offer `~/.config/x -> ~/.local/state/x` as a declaration to adopt.
    if source.starts_with(home) {
        return None;
    }
    if !in_a_repository(&source, memo) {
        return None;
    }
    Some(ManagedLink {
        source,
        target: path.to_path_buf(),
    })
}

/// `~/…` for a path under `home`, absolute otherwise.
///
/// The manifest is written to be read on another machine, so `@target` is spelled the way `~`
/// is: a declaration naming `/home/alice/.vimrc` is one no second machine can use.
pub fn tilde(home: &Path, path: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

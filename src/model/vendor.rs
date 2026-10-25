//! `shall add` — vendoring someone else's modules into your repo (XIII.14, U14).
//!
//! `use` takes a name, never a URL (target-state): consuming a stranger's module is a **fetch
//! step** that puts files on disk, and then you `use` them by name like everything else. This
//! is that fetch step.
//!
//! **Vendoring, not importing.** The files land in your repo as a reviewable git diff — the one
//! thing between you and a stranger's code. It is a real defence and a weak one (nobody reads a
//! whole diff), so it is not the *only* defence: anything the vendored files can execute — an
//! `exec:` verb, an `adapters/*.toml` backend definition — arrives **unapproved**, and II.12's
//! ledger holds it until `shall lock`. Refuse-by-default, one deliberate act to permit, is the
//! pattern the guard's `--allow-mass-removal`, dotfiles' `--replace-existing` and `@allow_http`
//! all already follow. `--trust` vendors and locks in one step, for a source you already trust.
//!
//! Pure: classifying a source, keeping a fetched path inside the repo, and planning what lands
//! where (refusing a collision). The fetch and the copy are the caller's.

use std::path::{Path, PathBuf};

/// Where a module is being vendored from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// `github:owner/repo` — the shorthand, expanded to a clone URL.
    Github { owner: String, repo: String },
    /// Any git remote — `https://…`, `git@…`, `ssh://…`.
    Git(String),
    /// A single raw file over http(s).
    File(String),
    /// A module file or repo already on disk.
    Local(PathBuf),
}

impl Source {
    /// Classify what the user typed. The order is deliberate: `github:` is checked before the
    /// generic git test, and a `.git` URL or a git scheme before a plain file URL, so a git
    /// remote is never mistaken for a single file to download.
    pub fn classify(input: &str) -> Option<Source> {
        let s = input.trim();
        if s.is_empty() {
            return None;
        }

        if let Some(rest) = s.strip_prefix("github:") {
            let (owner, repo) = rest.split_once('/')?;
            let repo = repo.trim_end_matches(".git").trim_end_matches('/');
            if owner.is_empty() || repo.is_empty() || repo.contains('/') || repo.contains('\\') {
                return None;
            }
            return Some(Source::Github {
                owner: owner.to_string(),
                repo: repo.to_string(),
            });
        }

        let is_url = s.starts_with("http://") || s.starts_with("https://");
        if s.starts_with("git@") || s.starts_with("ssh://") || (is_url && s.ends_with(".git")) {
            return Some(Source::Git(s.to_string()));
        }
        if is_url {
            return Some(Source::File(s.to_string()));
        }

        // Anything else is a path on this machine. A relative path is left relative; the caller
        // resolves it against the working directory.
        Some(Source::Local(PathBuf::from(s)))
    }

    /// The git clone URL, for the two source kinds that clone.
    pub fn clone_url(&self) -> Option<String> {
        match self {
            Source::Github { owner, repo } => {
                Some(format!("https://github.com/{}/{}.git", owner, repo))
            }
            Source::Git(url) => Some(url.clone()),
            _ => None,
        }
    }

    /// A short label naming the source, for messages and for namespacing if it were ever wanted.
    pub fn label(&self) -> String {
        match self {
            Source::Github { owner, repo } => format!("github:{}/{}", owner, repo),
            Source::Git(url) => url.clone(),
            Source::File(url) => url.clone(),
            Source::Local(p) => p.display().to_string(),
        }
    }
}

/// The kinds of file `shall add` will copy out of a fetched source.
///
/// **A closed list, deliberately.** A shared repo is a config repo, and only some of it is
/// yours to take: the reusable lists (`modules/`) and the definitions they may lean on
/// (`adapters/`, and scripts an `exec:` points at). Not `profiles/`, `active` or `priority` —
/// those are the *other* machine's choices, and importing them would silently reconfigure
/// yours. Copying everything is how a vendor step turns into "run this repo".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendored {
    /// A `modules/*.txt` list.
    Module,
    /// An `adapters/*.toml` definition — code, so it lands unapproved.
    Adapter,
    /// A `scripts/*` file an `exec:` line may reference — also code, also unapproved.
    Script,
}

impl Vendored {
    /// Which category a source-relative path falls into, or `None` for a file `add` leaves
    /// behind (a README, a `profiles/` file, the other machine's `active`).
    pub fn of(rel: &Path) -> Option<Vendored> {
        let mut comps = rel.components();
        let top = comps.next()?.as_os_str().to_str()?;
        let ext = rel.extension().and_then(|e| e.to_str());
        match (top, ext) {
            ("modules", Some("txt")) => Some(Vendored::Module),
            ("adapters", Some("toml")) => Some(Vendored::Adapter),
            ("scripts", _) => Some(Vendored::Script),
            _ => None,
        }
    }

    /// Whether files of this kind arrive as executable code, held by II.12 until `shall lock`.
    pub fn is_code(self) -> bool {
        matches!(self, Vendored::Adapter | Vendored::Script)
    }
}

/// One file the vendor step would place, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub kind: Vendored,
    /// Path inside the fetched source.
    pub from: PathBuf,
    /// Path inside the user's repo.
    pub to: PathBuf,
}

/// The result of planning a vendor: what would land, and what already exists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VendorPlan {
    pub placements: Vec<Placement>,
    /// Destinations that already exist — the collision the user is refused on (U14: refuse and
    /// name it, `--force` to overwrite).
    pub collisions: Vec<PathBuf>,
}

/// Keep a source-relative path inside the repo it is being copied into.
///
/// A fetched tree is a stranger's, and a file named `../../.bashrc` or `/etc/passwd` in it
/// would, joined naively, write outside the config repo entirely — the S-class path-traversal
/// this rejects. A path with a `..` component, an absolute path, or a Windows prefix is
/// refused; everything else is returned as the safe relative path.
pub fn safe_relative(rel: &Path) -> Option<PathBuf> {
    use std::path::Component;
    let mut clean = PathBuf::new();
    for comp in rel.components() {
        match comp {
            // A component the *running* platform calls ordinary may still be a separator on the
            // other one: Unix reads `..\..\x` as a single filename, Windows as a climb out. A
            // vendored tree is a stranger's, so the answer must not depend on which machine
            // fetched it — see `not_a_bare_file_name`, where the same asymmetry was live.
            Component::Normal(c) => match c.to_str() {
                Some(s) if s.contains('/') || s.contains('\\') => return None,
                _ => clean.push(c),
            },
            // A leading `./` is harmless noise; drop it.
            Component::CurDir => {}
            // Everything else — `..`, a root, a drive prefix — is an escape attempt.
            _ => return None,
        }
    }
    if clean.as_os_str().is_empty() {
        return None;
    }
    Some(clean)
}

/// Plan the vendor: for every file in `source_files` (paths relative to the fetched root),
/// decide what lands where, and flag the ones whose destination already exists.
///
/// `exists` answers whether a repo-relative destination is already present — injected so the
/// planner stays pure and testable without a repo on disk.
pub fn plan(source_files: &[PathBuf], exists: &dyn Fn(&Path) -> bool) -> VendorPlan {
    let mut plan = VendorPlan::default();
    for rel in source_files {
        let Some(kind) = Vendored::of(rel) else {
            continue;
        };
        let Some(safe) = safe_relative(rel) else {
            continue;
        };
        // The destination mirrors the source layout: a `modules/x.txt` lands at `modules/x.txt`.
        let to = safe.clone();
        if exists(&to) {
            plan.collisions.push(to.clone());
        }
        plan.placements.push(Placement {
            kind,
            from: rel.clone(),
            to,
        });
    }
    plan.placements.sort_by(|a, b| a.to.cmp(&b.to));
    plan.collisions.sort();
    plan
}

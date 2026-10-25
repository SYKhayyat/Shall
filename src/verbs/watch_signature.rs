//! What `shall watch` counts as a change, and how it says so.
//!
//! **It used to count one folder.** `manifest_signature` read `modules/*.txt` and compared size
//! and mtime, so `profiles/`, `active`, `priority`, `groups`, `schedules`, `vars/`, `adapters/`,
//! `preferences.toml` and the hooks could all change and a `watch --on-change` would sit there
//! reconciling nothing — the one failure a watcher cannot have, because it looks like the watcher
//! working. The file-name says what it looked at, which is how it was found: `manifest_signature`
//! over a `modules` directory, in a program that reads a dozen other files out of the same repo.
//!
//! **Content, hashed.** Size and mtime is a fingerprint that misses an edit which preserves both
//! — `git checkout` of a branch whose file happens to be the same length, an editor that rewrites
//! a file it did not change the length of, a filesystem with a coarse timestamp. A hash of the
//! bytes answers the only question there is, and it makes a `git pull` that brought nothing new a
//! non-event.
//!
//! **Two folders are not inputs, and they are excluded by name.** `.git/` is git's own
//! bookkeeping and a `pull` rewrites it on every tick whether or not your configuration moved.
//! `locks/` is generated *by the sync watch is running* — hash it and every tick writes the locks
//! that make the next tick see a change, which is a daemon that reconciles itself for ever. The
//! name is read from [`Layout`](crate::model::Layout) rather than spelled here, and a test holds
//! the two together, because a rename that moved one and not the other is a self-waking daemon.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// The file in the config repo that lists what `watch` should not react to.
///
/// **`.gitignore`'s shape, deliberately not `.gitignore`'s language.** One glob per line, `#` for a
/// comment, a trailing `/` for a directory, a leading `/` for the repo root, `**` across
/// directories, `*` within one. No negation (`!`), no character classes, no escape character —
/// a file whose language is a subset of a format everybody already knows is a file people can
/// predict, and the subset is small enough to state in the error a bad line produces.
pub const IGNORE_FILE: &str = ".shall-watchignore";

/// The two directories under the config root that are not inputs.
///
/// **`locks` is a name this module does not own.** [`NEVER_WATCHED`] holds it because spelling it
/// here would be a second copy of `Layout::locks_dir`, and the two drifting apart is a daemon that
/// wakes itself up.
pub const NEVER_WATCHED: &[&str] = &[".git", "locks"];

/// Everything under the config root that `watch` reads, and a hash of each one's bytes.
///
/// A `BTreeMap` rather than a `Vec` of tuples because the comparison is the whole point: two
/// passes are equal iff no byte of any input moved, and a map answers that without the caller
/// sorting or diffing by hand.
pub type Signature = BTreeMap<String, String>;

/// One pass over the config root.
pub async fn signature(root: &Path) -> Signature {
    let ignore = Ignore::read(root).await;
    let mut out = Signature::new();
    collect(root, root, &ignore, &mut out).await;
    out
}

/// What changed between two passes, as paths relative to the config root and sorted.
///
/// A **removed** file is in here too, and it has to be: deleting the last `profile` or the whole
/// `adapters/` directory is a change to what Shall does, and a diff that only reported new and
/// modified bytes would call that a non-event.
pub fn changed(before: &Signature, after: &Signature) -> Vec<String> {
    let mut names: Vec<String> = before
        .keys()
        .filter(|k| !after.contains_key(*k))
        .cloned()
        .collect();
    names.extend(
        after
            .iter()
            .filter(|(path, hash)| before.get(*path) != Some(*hash))
            .map(|(path, _)| path.clone()),
    );
    names.sort();
    names.dedup();
    names
}

/// The line `watch` prints when it reacts, naming what it reacted to.
///
/// **Five names and a count.** A `git pull` that brings a release's worth of churn is one event,
/// and an event that prints forty paths is a log line nobody reads — which is where it was before
/// it named anything. The count is still there, so the line never lies about how much moved.
pub fn say(names: &[String]) -> String {
    const NAMED: usize = 5;
    match names.split_first() {
        None => "nothing changed".to_string(),
        Some(_) if names.len() <= NAMED => names.join(", "),
        Some(_) => format!(
            "{} and {} more",
            names[..NAMED].join(", "),
            names.len() - NAMED
        ),
    }
}

fn collect<'a>(
    root: &'a Path,
    dir: &'a Path,
    ignore: &'a Ignore,
    out: &'a mut Signature,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        let Ok(mut rd) = tokio::fs::read_dir(dir).await else {
            return;
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(rel) = relative(root, &entry.path()) else {
                continue;
            };
            let Ok(kind) = entry.file_type().await else {
                continue;
            };
            if kind.is_dir() {
                // **The check is before the read, not after**: an ignored directory is never walked,
                // so `.git/` costs one `stat` rather than one walk of every object in the repo.
                if !NEVER_WATCHED.contains(&name.as_str()) && !ignore.matches(&rel, true) {
                    collect(root, &entry.path(), ignore, out).await;
                }
                continue;
            }
            if ignore.matches(&rel, false) {
                continue;
            }
            // **A symlink is hashed as the pointer, not as what it points at.** A symlinked file is a
            // real thing to re-point and a thing not to re-read: following it would make a change in
            // a directory outside the repo a change in the repo, and `watch` is a watcher of *this*
            // repo. The link's own text is the input, so the link's own text is the hash.
            let hash = if kind.is_symlink() {
                match tokio::fs::read_link(entry.path()).await {
                    Ok(target) => digest(target.to_string_lossy().as_bytes()),
                    Err(_) => continue,
                }
            } else if kind.is_file() {
                match tokio::fs::read(entry.path()).await {
                    Ok(bytes) => digest(&bytes),
                    Err(_) => continue,
                }
            } else {
                continue;
            };
            out.insert(rel, hash);
        }
    })
}

fn relative(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let text = rel.to_string_lossy().replace('\\', "/");
    (!text.is_empty()).then_some(text)
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// The user's own "never wake me for these".
struct Ignore {
    patterns: Vec<Pattern>,
}

impl Ignore {
    async fn read(root: &Path) -> Self {
        let Ok(text) = tokio::fs::read_to_string(root.join(IGNORE_FILE)).await else {
            return Self {
                patterns: Vec::new(),
            };
        };
        let mut bad = Vec::new();
        let patterns = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| match Pattern::parse(l) {
                Ok(p) => Some(p),
                Err(why) => {
                    bad.push(format!("  {l}: {why}"));
                    None
                }
            })
            .collect();
        if !bad.is_empty() {
            // **A line Shall cannot read is said and not skipped.** A pattern this module does not
            // understand is a file it will not honour, and honouring the others while ignoring
            // this one is how a user's ignore list quietly stops meaning what it says.
            tracing::warn!(
                "watch: {} has {} line(s) that are not a pattern, and they are NOT being ignored:\n{}",
                IGNORE_FILE,
                bad.len(),
                bad.join("\n")
            );
        }
        Self { patterns }
    }

    fn matches(&self, rel: &str, is_dir: bool) -> bool {
        self.patterns.iter().any(|p| p.matches(rel, is_dir))
    }
}

pub struct Pattern {
    /// Segments to match from the repo root, where `**` is the "any depth" segment.
    segments: Vec<String>,
    /// `name/` — the pattern names a directory, so a file of that name is not what it names.
    directory_only: bool,
    /// A leading `/`, or a `/` anywhere in the body: `README.md` at the root and
    /// `docs/README.md` are different things, and a pattern naming a path is about that path.
    anchored: bool,
}

impl Pattern {
    pub fn parse(line: &str) -> std::result::Result<Self, &'static str> {
        if line.starts_with('!') {
            return Err("negation (`!`) is not supported: a name is either ignored or not");
        }
        if line.contains('[') || line.contains(']') {
            return Err("character classes are not supported; write the names out");
        }
        let directory_only = line.ends_with('/');
        let trimmed = line.trim_end_matches('/');
        let body = trimmed.trim_start_matches('/');
        // **A slash anywhere in the pattern anchors it**, which is the rule `.gitignore` uses and
        // the one a pattern-with-a-path needs: `a/**/b` is a path from the repo root, so it must
        // not also match `z/a/b`. Only a pattern with *no* other slash floats, and then it floats
        // as a basename — `*.bak` at any level, which is what everybody expects of it.
        let anchored = trimmed.starts_with('/') || body.contains('/');
        if body.is_empty() {
            return Err("a pattern needs something to match");
        }
        Ok(Self {
            segments: body.split('/').map(str::to_string).collect(),
            directory_only,
            anchored,
        })
    }

    pub fn matches(&self, rel: &str, is_dir: bool) -> bool {
        if self.directory_only && !is_dir {
            return false;
        }
        let parts: Vec<&str> = rel.split('/').collect();
        if self.anchored {
            return match_here(&self.segments, &parts);
        }
        // Unanchored: the pattern may start anywhere, the way `.gitignore` reads.
        (0..parts.len()).any(|start| match_here(&self.segments, &parts[start..]))
    }
}

fn match_here(pattern: &[String], parts: &[&str]) -> bool {
    if pattern.is_empty() {
        return parts.is_empty();
    }
    if pattern[0] == "**" {
        // `**` absorbs zero or more segments, so `a/**/b` matches `a/b` as well as `a/x/y/b`.
        return (0..=parts.len()).any(|skip| match_here(&pattern[1..], &parts[skip..]));
    }
    if parts.is_empty() {
        return false;
    }
    glob_segment(&pattern[0], parts[0]) && match_here(&pattern[1..], &parts[1..])
}

/// `*` matches within one segment and `?` matches one character; nothing else is a wildcard.
fn glob_segment(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = pi;
            mark = ni;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

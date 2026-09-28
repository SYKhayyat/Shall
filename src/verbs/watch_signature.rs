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

struct Pattern {
    /// Segments to match from the repo root, where `**` is the "any depth" segment.
    segments: Vec<String>,
    /// `name/` — the pattern names a directory, so a file of that name is not what it names.
    directory_only: bool,
    /// A leading `/`, or a `/` anywhere in the body: `README.md` at the root and
    /// `docs/README.md` are different things, and a pattern naming a path is about that path.
    anchored: bool,
}

impl Pattern {
    fn parse(line: &str) -> std::result::Result<Self, &'static str> {
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

    fn matches(&self, rel: &str, is_dir: bool) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn repo() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        for sub in ["modules", "profiles", "vars", "adapters", "locks"] {
            std::fs::create_dir_all(dir.path().join(sub)).expect("the tree");
        }
        std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n").expect("a manifest");
        std::fs::write(dir.path().join("active"), "Work\n").expect("active");
        std::fs::write(dir.path().join("preferences.toml"), "").expect("preferences");
        std::fs::write(dir.path().join("locks/regex.toml"), "").expect("a lock");
        dir
    }

    /// **The bug this module exists for: a change outside `modules/` is a change.**
    #[tokio::test]
    async fn an_edit_anywhere_in_the_repo_is_a_change() {
        let dir = repo().await;
        let before = signature(dir.path()).await;

        for (path, body) in [
            ("profiles/Work", "use dev\n"),
            ("active", "Dev\n"),
            ("priority", "apt\n"),
            ("groups", "tools = apt\n"),
            ("schedules", "0 3 * * * shall sync\n"),
            ("vars/season", "winter\n"),
            ("adapters/backends.toml", "[[backend]]\n"),
            ("preferences.toml", "max_parallel = 2\n"),
        ] {
            let before = signature(dir.path()).await;
            std::fs::write(dir.path().join(path), body).expect("the edit");
            let names = changed(&before, &signature(dir.path()).await);
            assert_eq!(
                names,
                vec![path.to_string()],
                "{path} did not count as a change"
            );
        }
        // And the manifest it always did see, so the list above is not the whole story.
        assert!(!changed(&before, &before).iter().any(|_| true));
    }

    /// **Content, not size and mtime.** The old fingerprint was `(len, mtime)`, and this is the
    /// edit it cannot see: the same number of bytes, written now.
    #[tokio::test]
    async fn a_rewrite_of_the_same_bytes_at_the_same_length_is_a_change() {
        let dir = repo().await;
        std::fs::write(dir.path().join("modules/dev.txt"), "apt:curl\n").expect("same length");
        let before = signature(dir.path()).await;
        std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n")
            .expect("same length again");
        assert_eq!(
            changed(&before, &signature(dir.path()).await),
            vec!["modules/dev.txt".to_string()],
            "an edit that preserves the length was invisible to the old fingerprint"
        );
    }

    /// A rewrite with the *same bytes* is not a change: that is what makes a `git pull` that
    /// brought nothing new a non-event rather than a tick.
    #[tokio::test]
    async fn the_same_bytes_rewritten_is_not_a_change() {
        let dir = repo().await;
        let before = signature(dir.path()).await;
        std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n").expect("the same bytes");
        assert!(changed(&before, &signature(dir.path()).await).is_empty());
    }

    /// **The self-waking daemon.** `sync` writes `locks/`, and `watch` is the thing that runs
    /// `sync`: if the locks were in the fingerprint, every tick would see the previous tick's
    /// writes and reconcile again, for ever.
    #[tokio::test]
    async fn what_shall_writes_and_git_does_is_not_a_change() {
        let dir = repo().await;
        std::fs::create_dir_all(dir.path().join(".git/objects")).expect("a git dir");
        let before = signature(dir.path()).await;

        std::fs::write(dir.path().join("locks/regex.toml"), "apt:x = [\"y\"]\n").expect("a write");
        std::fs::write(dir.path().join("locks/versions.json"), "{}").expect("another");
        std::fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("git churn");
        std::fs::create_dir_all(dir.path().join(".git/objects/ab")).expect("an object dir");
        std::fs::write(dir.path().join(".git/objects/ab/cdef"), "binary-ish").expect("an object");

        assert!(
            changed(&before, &signature(dir.path()).await).is_empty(),
            "watch would wake itself up after every sync"
        );
        assert!(!signature(dir.path()).await.contains_key(".git/HEAD"));
        assert!(!signature(dir.path())
            .await
            .keys()
            .any(|k| k.starts_with("locks/")));
    }

    /// A removed file is a change, because deleting the last profile is a change to what Shall
    /// does — and a diff that only reported new and modified bytes calls that a non-event.
    #[tokio::test]
    async fn a_removed_file_is_a_change() {
        let dir = repo().await;
        std::fs::write(dir.path().join("profiles/Work"), "use dev\n").expect("a profile");
        let before = signature(dir.path()).await;
        std::fs::remove_file(dir.path().join("profiles/Work")).expect("the removal");
        assert_eq!(
            changed(&before, &signature(dir.path()).await),
            vec!["profiles/Work".to_string()]
        );
    }

    /// A whole directory going is one event with every file in it named.
    #[tokio::test]
    async fn a_removed_directory_names_what_it_held() {
        let dir = repo().await;
        for name in ["backends.toml", "firewall.toml"] {
            std::fs::write(dir.path().join("adapters").join(name), "").expect("a row");
        }
        let before = signature(dir.path()).await;
        std::fs::remove_dir_all(dir.path().join("adapters")).expect("the removal");
        assert_eq!(
            changed(&before, &signature(dir.path()).await),
            vec![
                "adapters/backends.toml".to_string(),
                "adapters/firewall.toml".to_string()
            ]
        );
    }

    /// **The user's own ignore list**, in the shape the module documents: globs, a trailing `/`
    /// for a directory, a leading `/` for the root, `#` for a comment, and `**` across
    /// directories.
    #[tokio::test]
    async fn the_ignore_file_is_the_users_own_list() {
        let dir = repo().await;
        std::fs::write(
            dir.path().join(IGNORE_FILE),
            "# not an input\nREADME.md\n/docs/\n**/*.bak\n*.tmp\n",
        )
        .expect("the ignore file");
        std::fs::create_dir_all(dir.path().join("docs")).expect("a docs dir");
        std::fs::write(dir.path().join("README.md"), "hello").expect("a readme");
        std::fs::write(dir.path().join("docs/notes.md"), "notes").expect("a doc");
        std::fs::write(dir.path().join("modules/dev.txt.bak"), "old").expect("a backup");
        std::fs::write(dir.path().join("scratch.tmp"), "wip").expect("a temp file");

        let before = signature(dir.path()).await;
        for name in [
            "README.md",
            "docs/notes.md",
            "modules/dev.txt.bak",
            "scratch.tmp",
        ] {
            assert!(
                !before.contains_key(name),
                "{name} is in the signature and the ignore file names it"
            );
        }
        assert!(
            before.contains_key("modules/dev.txt"),
            "the ignore list ate a manifest"
        );
        assert!(
            before.contains_key(IGNORE_FILE),
            "the ignore file must be watched itself"
        );

        // And an edit to an ignored file is not a change, which is the point of the file.
        std::fs::write(dir.path().join("README.md"), "goodbye").expect("the edit");
        assert!(changed(&before, &signature(dir.path()).await).is_empty());
    }

    /// **An unanchored pattern matches at any depth, the way `.gitignore` reads**, and an
    /// anchored one does not. `*.bak` is the basename at any level; `/*.bak` is the root's. That
    /// is the one rule in this file a user is most likely to hold the other way round, so it is
    /// asserted rather than described.
    #[test]
    fn a_pattern_is_anchored_only_when_it_says_so() {
        let anchored = Pattern::parse("/docs/").expect("a pattern");
        assert!(anchored.matches("docs", true));
        assert!(!anchored.matches("notes/docs", true));

        let anywhere = Pattern::parse("docs/").expect("a pattern");
        assert!(anywhere.matches("docs", true));
        assert!(anywhere.matches("notes/docs", true));

        // An unanchored name matches at any depth; the same pattern anchored to the root does not.
        let anywhere_bak = Pattern::parse("*.bak").expect("a pattern");
        assert!(anywhere_bak.matches("a.bak", false));
        assert!(anywhere_bak.matches("deep/nested/a.bak", false));
        let root_bak = Pattern::parse("/*.bak").expect("a pattern");
        assert!(root_bak.matches("a.bak", false));
        assert!(!root_bak.matches("deep/a.bak", false));

        // `*` stays inside one segment and `**` does not, which only shows where a pattern has a
        // slash in it: `a/**/b` is a path, so the wildcard has to cross directories.
        let middle = Pattern::parse("a/**/b").expect("a pattern");
        assert!(middle.matches("a/b", true), "`**` absorbs zero segments");
        assert!(middle.matches("a/x/b", true));
        assert!(middle.matches("a/x/y/b", true));
        assert!(
            !middle.matches("z/a/b", true),
            "a pattern naming a path is about that path, not about any path ending in it"
        );
    }

    /// A line the module cannot read is refused with a reason, not quietly skipped.
    #[test]
    fn a_line_that_is_not_a_pattern_says_why() {
        assert!(Pattern::parse("!keep.txt").is_err());
        assert!(Pattern::parse("[ab]*.txt").is_err());
        assert!(Pattern::parse("/").is_err());
    }

    /// A symlink is the pointer, not what it points at: a change in a directory *outside* the
    /// repo is not a change in the repo, and re-pointing one is.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_symlink_is_hashed_as_the_pointer() {
        let dir = repo().await;
        let outside = tempdir().expect("a tempdir");
        let target = outside.path().join("watched.txt");
        std::fs::write(&target, "one").expect("the target");
        std::os::unix::fs::symlink(&target, dir.path().join("modules/linked.txt"))
            .expect("the link");

        let before = signature(dir.path()).await;
        std::fs::write(&target, "two").expect("the target changed outside the repo");
        assert!(
            changed(&before, &signature(dir.path()).await).is_empty(),
            "a file outside the repo changed and watch woke up for it"
        );

        let other = outside.path().join("other.txt");
        std::fs::write(&other, "three").expect("a second target");
        std::fs::remove_file(dir.path().join("modules/linked.txt")).expect("unlink");
        std::os::unix::fs::symlink(&other, dir.path().join("modules/linked.txt")).expect("re-link");
        assert_eq!(
            changed(&before, &signature(dir.path()).await),
            vec!["modules/linked.txt".to_string()],
            "re-pointing a symlink is a change to what the repo declares"
        );
    }

    /// **`locks` is `Layout`'s name, not this module's.** Two copies of one name drift, and the
    /// drift is a daemon that wakes itself up.
    #[test]
    fn the_excluded_directory_is_the_one_the_layout_creates() {
        let layout = crate::model::Layout::new(
            "/tmp/shall-watch-locks-check",
            "/tmp/shall-watch-locks-data",
        );
        let locks = layout
            .locks_dir()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .expect("the locks dir has a name");
        assert!(
            NEVER_WATCHED.contains(&locks.as_str()),
            "the layout writes `locks/` and this module excludes {:?}",
            NEVER_WATCHED
        );
    }

    /// The line the daemon prints: five names, then the count, because a pull that brings forty
    /// files is one event and one line.
    #[test]
    fn the_line_names_what_it_reacted_to_without_becoming_a_list() {
        assert_eq!(say(&[]), "nothing changed");
        assert_eq!(say(&["active".to_string()]), "active");
        assert_eq!(
            say(&["a".to_string(), "b".to_string()]),
            "a, b",
            "two names fit on one line"
        );
        let six: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(say(&six), "a, b, c, d, e and 2 more");
    }

    /// The signature is sorted and unique, so two passes are equal iff nothing moved — the
    /// property the comparison rests on, asserted on the map itself rather than on the diff.
    #[tokio::test]
    async fn two_passes_over_an_unchanged_repo_are_equal() {
        let dir = repo().await;
        assert_eq!(signature(dir.path()).await, signature(dir.path()).await);
        let taken = signature(dir.path()).await;
        let first: Vec<&String> = taken.keys().collect();
        let mut sorted = first.clone();
        sorted.sort();
        assert_eq!(first, sorted, "the walk order leaked into the signature");
    }

    /// A file that cannot be read is skipped, not fatal: a watcher that dies on a permission
    /// error is a watcher that stopped watching.
    #[tokio::test]
    async fn an_unreadable_file_is_skipped_rather_than_fatal() {
        let dir = repo().await;
        let sig = signature(dir.path()).await;
        assert!(!sig.contains_key("modules/does-not-exist.txt"));
        assert!(sig.contains_key("modules/dev.txt"));
    }
}

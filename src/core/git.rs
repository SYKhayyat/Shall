// Version control for Shall's *intent* — the manifest/config directory.
//
// Filesystem snapshots version the *effect* of a change — the whole disk. Git is the other
// half and the complementary one: the human-readable, diffable, branchable, pushable history
// of what you *asked for* (II.13). `git diff` shows "you added ripgrep, removed nano"; a
// remote backs your whole setup up like dotfiles. There is no generation format; a generation
// IS a commit.
//
// This is a thin, dependency-free wrapper that shells out to the system `git` (Shall already
// shells out to every package manager, so this adds no new dependency and no libgit2 build
// cost). Every method that could fail on a machine without git returns a `Result` the caller
// can degrade gracefully on — auto-commit, for instance, simply no-ops when git is absent.
//
// The repo root is the Shall config directory, so a single repo captures `preferences.toml`,
// `modules/`, `profiles/`, `active`, `priority` and `locks/` together.

use crate::core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Output;

/// What git says about a commit's signature (II.13). Shall verifies nothing itself: `git`
/// answers, and this is its answer carried without interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    /// No signature at all — the state of every commit in a repo nobody signs.
    Unsigned,
    /// A good signature, by this signer.
    Good(String),
    /// A signature git could read but will not vouch for: an untrusted, expired or revoked
    /// key. Kept apart from `Good` — "signed" and "signed by someone you trust" are different
    /// claims, and collapsing them is how a signature becomes decoration (V.32).
    Unverified { signer: String, code: char },
    /// A signature that does not match the commit, or that git could not check at all.
    Bad,
}

impl Signature {
    /// `%G?` is git's own verdict; `%GS` is the signer it read. Any code git adds later lands
    /// in `Unverified` rather than being read as good.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn from_git(code: &str, signer: &str) -> Self {
        match code.chars().next().unwrap_or('N') {
            'N' => Self::Unsigned,
            'G' => Self::Good(signer.to_string()),
            'B' | 'E' => Self::Bad,
            other => Self::Unverified {
                signer: signer.to_string(),
                code: other,
            },
        }
    }

    /// Whether git vouches for this commit. Only a good signature does.
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Good(_))
    }

    /// One line for a log row or a refusal.
    pub fn describe(&self) -> String {
        match self {
            Self::Unsigned => "unsigned".to_string(),
            Self::Good(signer) => format!("signed by {}", signer),
            Self::Unverified { signer, code } => {
                format!(
                    "signed by {} — git will not vouch for it ({})",
                    signer, code
                )
            }
            Self::Bad => "a bad signature, or one git could not check".to_string(),
        }
    }
}

/// A commit as shown by `git log` — the data `shall git log` renders. A generation IS a
/// commit (II.13), so this is the whole of the history record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommit {
    pub hash: String,
    pub short: String,
    pub date: String,
    pub subject: String,
    pub signature: Signature,
}

/// A git wrapper scoped to one directory (the Shall config root).
#[derive(Debug, Clone)]
pub struct GitManager {
    root: PathBuf,
}

impl GitManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn git_available() -> bool {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("--version").stdin(std::process::Stdio::null());
        crate::core::blocking::command_output(&mut cmd)
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// git itself, asked before anything about this directory.
    ///
    /// "No history yet" is a normal state that reads as success, and [`is_repo`] answers it
    /// from the filesystem alone — so without this a missing git wears that answer: `git log`
    /// printed an empty history and `git status` advised running `git init`, which cannot
    /// succeed. X.5 keeps git optional; it does not make its absence an empty answer.
    ///
    /// [`is_repo`]: Self::is_repo
    pub fn require() -> Result<()> {
        if Self::git_available() {
            return Ok(());
        }
        Err(Error::Other(
            "git is not installed; install it to use Shall's manifest history — \
             `shall git`, `diff`, `rollback` and `bundle`. Everything else works without it."
                .into(),
        ))
    }

    pub fn is_repo(&self) -> bool {
        self.root.join(".git").exists()
    }

    /// Stop git asking a question nobody can answer (`S88`'s family).
    ///
    /// **ssh is deliberately left alone.** `-o BatchMode=yes` would close the last hole, but the
    /// only way to set it from here is `GIT_SSH_COMMAND`, which overrides a user's
    /// `core.sshCommand` — so silencing a prompt on a misconfigured remote would break every
    /// working custom transport. A passphrase with no agent is the user's setup to fix; an
    /// unprompted credential is ours.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn ask_nothing(cmd: &mut std::process::Command) {
        // git's own prompt.
        cmd.env("GIT_TERMINAL_PROMPT", "0");
        // Git Credential Manager, the default on Windows, which does not print a prompt — it
        // opens a *window*, so nothing appears in a captured stream at all and the wait looks
        // exactly like a slow network.
        cmd.env("GCM_INTERACTIVE", "never");
    }

    /// Nothing about the user's git is overridden here — not the identity, not the signing
    /// flags. A commit signed by your key and authored by `shall@localhost` attributes a
    /// verified change to a person who does not exist (owner ruling, 2026-07-21), and a repo
    /// with no identity configured is git's error to report, in git's own words.
    fn run(&self, args: &[&str]) -> Result<Output> {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("-C").arg(&self.root);
        cmd.args(args);
        // git prompts — for a credential, for a passphrase — and this captures both its
        // streams, so the question would be asked where nobody can read it.
        //
        // **Closing stdin does not stop it, and believing it did was the same mistake as
        // `S88`.** git's credential prompt, like sudo's, is read from `/dev/tty` and not from
        // stdin, so a null stdin leaves a session with a controlling terminal waiting for an
        // answer nobody is there to give. These two variables are what actually turn the
        // question into an error: git declines to prompt, and Git Credential Manager — the
        // default on Windows, which pops a *window* — declines to open one.
        cmd.stdin(std::process::Stdio::null());
        Self::ask_nothing(&mut cmd);
        // Blocking, from a synchronous API that async verbs call — and Shall runs git after
        // every successful sync. Held a runtime worker for the length of every commit.
        crate::core::blocking::command_output(&mut cmd)
            .map_err(|e| Error::command_failed(format!("git {:?} failed to spawn: {}", args, e)))
    }

    fn run_checked(&self, args: &[&str]) -> Result<String> {
        let out = self.run(args)?;
        if !out.status.success() {
            return Err(Error::command_failed(format!(
                "git {:?}: {}",
                args,
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &out.stdout,
        )))
    }

    /// A non-fast-forward or missing remote surfaces as an error the caller can downgrade
    /// to a warning — `watch --pull` must not abort a sync because a remote moved.
    pub fn pull(&self) -> Result<String> {
        Self::require()?;
        self.run_checked(&["pull", "--ff-only"])
    }

    /// Idempotent. The written `.gitignore` excludes the per-file backups Shall drops
    /// during rollbacks, which would otherwise be committed as manifest content.
    pub fn init(&self) -> Result<()> {
        Self::require()?;
        // A preview creates no repository and makes no commit. `--dry-run git init` used to do
        // both, which is the one case in this family where the preview's side effect is a
        // permanent artifact in the user's config directory rather than a changed file.
        if crate::core::dry_run::active() {
            crate::would_warn!(
                "would initialise manifest version control at {} and commit the \
                 config as it stands.",
                self.root.display()
            );
            return Ok(());
        }
        crate::utils::file::ensure_dir(&self.root)?;
        if !self.is_repo() {
            self.run_checked(&["init"])?;
        }
        let ignore = self.root.join(".gitignore");
        // A pattern the user has commented out is not a pattern they have, which is why this
        // reads the meaningful lines rather than searching the text.
        let present = crate::utils::file::read_lines_filtered(&ignore)?;
        // One pattern, asked for by name rather than by a one-element `for`. **The loop was
        // written as a loop because a second pattern was expected**, and it has been the only
        // place a lint that only the advisory job runs could reach — `single_element_loop`, on
        // code this commit did not touch, when clippy 1.99 added it. If a second pattern arrives
        // this becomes a loop again, and the named constant is where it arrives.
        const BACKUP_GLOB: &str = "*.shall-backup";
        if !present.iter().any(|l| l == BACKUP_GLOB) {
            crate::utils::file::append_line(&ignore, BACKUP_GLOB)?;
        }
        Ok(())
    }

    /// Stage everything and commit. Returns `Ok(Some(hash))` when a commit was created, or
    /// `Ok(None)` when there was nothing to commit (a clean tree — not an error). Callers use
    /// the `None` case to stay quiet on no-op runs.
    pub fn commit_all(&self, message: &str) -> Result<Option<String>> {
        // Every auto-commit after a command comes through here too, so a preview that changed
        // nothing has nothing to record — and one that would have changed something must not
        // write the record either.
        if crate::core::dry_run::active() {
            crate::would!("would commit: {}", message);
            return Ok(None);
        }
        if !self.is_repo() {
            return Err(Error::Other(format!(
                "{} is not a git repo; run `shall git init` first",
                self.root.display()
            )));
        }
        self.run_checked(&["add", "-A"])?;
        // If the index has no staged changes, `git commit` exits non-zero. Detect that
        // cleanly rather than surfacing it as a failure.
        let status = self.run_checked(&["status", "--porcelain"])?;
        if status.is_empty() {
            return Ok(None);
        }
        let out = self.run(&["commit", "-m", message])?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(Error::command_failed(format!(
                "git refused to commit your manifests:\n\n{}\n\n{}",
                stderr,
                commit_refusal_hint(&stderr)
            )));
        }
        Ok(Some(self.head()?.unwrap_or_default()))
    }

    /// The current HEAD commit hash, or `None` if the repo has no commits yet.
    pub fn head(&self) -> Result<Option<String>> {
        if !self.is_repo() {
            return Ok(None);
        }
        let out = self.run(&["rev-parse", "HEAD"])?;
        if out.status.success() {
            Ok(Some(crate::utils::text::sanitize(
                &String::from_utf8_lossy(&out.stdout),
            )))
        } else {
            // `rev-parse HEAD` also fails for a damaged repository. Only an unborn symbolic
            // branch is the normal no-commit case; every other failure must remain visible so
            // history consumers do not mistake corruption for an empty history.
            let branch = self.run(&["symbolic-ref", "--quiet", "--short", "HEAD"])?;
            if branch.status.success() {
                Ok(None)
            } else {
                Err(Error::command_failed(format!(
                    "git cannot read HEAD in {}: {}",
                    self.root.display(),
                    String::from_utf8_lossy(&out.stderr).trim()
                )))
            }
        }
    }

    /// The content of a tracked file as of HEAD, or `None` when the file was not tracked there
    /// or the repo has no commits yet. Since Shall commits only on a successful sync (V.30),
    /// HEAD is the last-synced state — the baseline for showing what a working-tree edit changed.
    pub fn show_at_head(&self, relpath: &str) -> Result<Option<String>> {
        if self.head()?.is_none() {
            return Ok(None);
        }
        let out = self.run(&["show", &format!("HEAD:{}", relpath)])?;
        if out.status.success() {
            Ok(Some(String::from_utf8_lossy(&out.stdout).to_string()))
        } else {
            Ok(None)
        }
    }

    /// Restore the working tree of the config directory to a given commit/ref WITHOUT moving
    /// HEAD — i.e. roll back your *manifests* to a past state, leaving installed packages
    /// untouched. This is the "config half" of a rollback.
    pub fn checkout_files(&self, reference: &str) -> Result<()> {
        Self::require()?;
        if !self.is_repo() {
            return Err(Error::Other("not a git repo".into()));
        }
        self.run_checked(&["checkout", reference, "--", "."])?;
        Ok(())
    }

    /// The most recent `limit` commits, newest first.
    pub fn log(&self, limit: usize) -> Result<Vec<GitCommit>> {
        Self::require()?;
        if !self.is_repo() || self.head()?.is_none() {
            return Ok(vec![]);
        }
        // Unit-separated fields, record-separated lines — robust against subjects with spaces.
        // The subject goes last: it is the only field that could contain the separator.
        let fmt = "--pretty=format:%H%x1f%h%x1f%cs%x1f%G?%x1f%GS%x1f%s";
        let raw = self.run_checked(&["log", &format!("-{}", limit.max(1)), fmt])?;
        Ok(parse_log(&raw))
    }

    /// What git says about one commit's signature (II.13). Asked of the commit a rollback is
    /// about to restore, so the answer is about that commit and not about HEAD.
    pub fn signature_of(&self, reference: &str) -> Result<Signature> {
        Self::require()?;
        let raw = self.run_checked(&["log", "-1", "--pretty=format:%G?%x1f%GS", reference])?;
        let mut parts = raw.split('\u{1f}');
        Ok(Signature::from_git(
            parts.next().unwrap_or("N"),
            parts.next().unwrap_or("").trim(),
        ))
    }

    /// Short status (porcelain) of the config repo, or an empty string if clean.
    pub fn status_porcelain(&self) -> Result<String> {
        if !self.is_repo() {
            return Err(Error::Other("not a git repo".into()));
        }
        self.run_checked(&["status", "--porcelain"])
    }

    /// The manifest lines a commit added or removed — the package-level story of that commit.
    /// `git show` limited to the config files, keeping only the
    /// `+`/`-` content lines (diff headers and comments dropped). Empty for a commit that
    /// touched no manifests. This is what replaced the generation format's stored package sets:
    /// git already records exactly what each change did to your manifests.
    pub fn commit_manifest_changes(&self, reference: &str) -> Result<Vec<String>> {
        if !self.is_repo() {
            return Ok(vec![]);
        }
        let raw = self.run_checked(&[
            "show",
            "--format=",
            "--no-color",
            reference,
            "--",
            "modules",
            "profiles",
            "active",
            "priority",
            "schedules",
            // `vars`, `vars.shall`, `vars.py` … — the file that explains a change must be in the
            // change view, or a variable edit that removed a hundred packages is invisible (W14).
            "vars*",
        ])?;
        Ok(parse_manifest_changes(&raw))
    }

    /// Write a `git bundle` of the whole repo to `dest` — every commit and ref in one file,
    /// for an air-gapped transfer. `git clone <dest>` on the far side reconstructs the repo
    /// with its full history, so the recipient can `rollback` to any past commit, not just
    /// restore the current manifests. Returns `Ok(false)` (nothing written) when there is no
    /// repo or no commits yet — a bundle honestly reports what it could not include.
    /// Whether a [`GitManager::bundle`] would have anything to carry. The precondition, asked
    /// without the side effect — a preview needs the answer and must not produce the file.
    pub fn has_commits(&self) -> bool {
        self.is_repo() && self.head().ok().flatten().is_some()
    }

    pub fn bundle(&self, dest: &Path) -> Result<bool> {
        if !self.has_commits() {
            return Ok(false);
        }
        let dest = dest.to_string_lossy().to_string();
        self.run_checked(&["bundle", "create", &dest, "--all"])?;
        Ok(true)
    }

    /// The manifest lines that differ between two commits — `shall diff <from> <to>` in
    /// packages, not text (Phase 4). `from` is the older baseline; pass `to = None` to diff
    /// `from` against the working tree (committed + uncommitted). Limited to the config files,
    /// keeping only the `+`/`-` content lines. Because manifests are package declarations, this
    /// diff IS the package-level story: what you'd add or remove going from one to the other.
    pub fn diff_manifest_changes(&self, from: &str, to: Option<&str>) -> Result<Vec<String>> {
        if !self.is_repo() {
            return Ok(vec![]);
        }
        let range = match to {
            Some(to) => format!("{}..{}", from, to),
            None => from.to_string(),
        };
        let raw = self.run_checked(&[
            "diff",
            "--no-color",
            &range,
            "--",
            "modules",
            "profiles",
            "active",
            "priority",
            "schedules",
            "vars*",
        ])?;
        Ok(parse_manifest_changes(&raw))
    }
}

/// What to do about a commit git refused. The two reachable causes are configuration, and both
/// became reachable when Shall stopped injecting an identity and stopped forcing signing off —
/// git's own message names the problem, this names the fix in Shall's terms.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn commit_refusal_hint(stderr: &str) -> &'static str {
    let lower = stderr.to_lowercase();
    if lower.contains("tell me who you are") || lower.contains("empty ident") {
        return "Shall commits as you, not as itself, so git needs an identity:\n  \
                git config --global user.name  \"Your Name\"\n  \
                git config --global user.email \"you@example.com\"";
    }
    if lower.contains("gpg failed") || lower.contains("failed to sign") {
        return "Your git is set to sign commits (`commit.gpgsign`) and the signing key did not \
                work. Fix the key, or turn signing off for this repo with \
                `git config commit.gpgsign false`.";
    }
    "Your manifests are unchanged on disk; the history simply did not record this run."
}

/// Extract the `+`/`-` content lines from a `git show` diff — the added and removed manifest
/// lines — skipping the `+++`/`---` file headers and blank/comment lines.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn parse_manifest_changes(diff: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in diff.lines() {
        let keep_plus = line.starts_with('+') && !line.starts_with("+++");
        let keep_minus = line.starts_with('-') && !line.starts_with("---");
        if keep_plus || keep_minus {
            let (sign, body) = line.split_at(1);
            let body = body.trim();
            if !body.is_empty() && !body.starts_with('#') {
                out.push(format!("{} {}", sign, body));
            }
        }
    }
    out
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn parse_log(raw: &str) -> Vec<GitCommit> {
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            let hash = parts.next()?.to_string();
            let short = parts.next()?.to_string();
            let date = parts.next()?.to_string();
            let code = parts.next().unwrap_or("N");
            let signer = parts.next().unwrap_or("").trim().to_string();
            let subject = parts.next().unwrap_or("").to_string();
            Some(GitCommit {
                hash,
                short,
                date,
                subject,
                signature: Signature::from_git(code, &signer),
            })
        })
        .collect()
}

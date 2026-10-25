use crate::core::{Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};
use tracing::warn;

/// The allowlist must stay wide enough for names that are legitimately not bare words:
/// npm `@scope`, github `owner/repo`, versioned `pkg:1.2+build`.
static PACKAGE_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._@:+/-]+$").unwrap());

/// The same allowlist plus the characters a Windows package identifier is built from.
///
/// `winget list` reports 185 of 278 names on a stock box as `ARP\Machine\X64\...` or
/// `MSIX\...`, and the ARP rows for MSI installers are GUIDs in braces. Those are the
/// identifiers `winget install` and `winget uninstall` take, so they are the names Shall has to
/// be able to carry (V.113).
///
/// **The space is here for the same reason the backslash is.** `ARP\Machine\X64\Mozilla Firefox`
/// is what winget answers and what winget accepts back; a rule that admits the backslash and
/// stops at the space carries most of an identifier and not the one the user has. It is safe
/// for the reason stated below: a name is argv, never a shell string. The grammar makes such a
/// name writable by quoting it, and this is the second half of V.113 — a name is admitted by a
/// grammar **and** a validator, and the two must agree.
///
/// **Inside the name, never at its edge.** A trailing space is invisible in a manifest, in a
/// listing and in every error message, so `Firefox ` and `Firefox` are one package that reads
/// as itself twice and reconciles as two. The grammar refuses the same shape at the same
/// boundary; a rule enforced on one side of a round trip is a rule the other side breaks.
static WINDOWS_IDENTIFIER_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    // The space is the only character legal in the middle and not at either end, so it is the
    // only one lifted out of the shared class.
    const INNER: &str = r"a-zA-Z0-9._@:+/\\{}-";
    Regex::new(&format!("^[{INNER}](?:[ {INNER}]*[{INNER}])?$")).unwrap()
});

/// Shell metacharacters, minus the three a Windows package identifier is made of.
///
/// Safe because **no package-manager command is ever a shell string** — every one is argv, and
/// that is the property the executor's own tests exist to keep. This list is defence in depth
/// against a name reaching a shell that does not exist; it is not what stands between a crafted
/// name and a command line.
static SHELL_INJECTION_REGEX_WINDOWS_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[;&|><`$\(\)\[\]\*\?\!]").unwrap());

/// Shell metacharacters blocked to prevent command injection.
static SHELL_INJECTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[;&|><`$\(\)\[\]\{\}\*\?\!\\]").unwrap());

/// The characters that let a string stop being one argument and start being a second command.
///
/// Deliberately narrower than [`SHELL_INJECTION_REGEX`], which is the metacharacter half of a
/// package-*name* rule and also bans `\`, `{}`, `*` and `?`. A winget identifier carries the
/// first two and an ordinary search query carries the last two, so that rule cannot be applied
/// to a string whose kind is not yet known. This one refuses the separators and nothing else:
///
/// - `& | < > ^` — `cmd` parses these out of its own command line before a `.cmd` shim sees
///   them. A package name of `q&calc.exe&rem` reaching a `.cmd` shim ran `calc.exe`.
/// - ``; $ ` ( )`` — the POSIX separators and substitutions. Shall never builds a shell string,
///   and the point of a second layer is to survive the day one of its callers does.
/// - `"` and the line terminators, which end an argument inside whatever is quoting it.
static COMMAND_METACHARACTER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("[;&|<>^`$()\"\r\n]").unwrap());

/// Sensitive system paths that Shall is prohibited from accessing.
static FORBIDDEN_PATHS: &[&str] = &[
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/passwd",
    "/etc/gshadow",
    "C:\\Windows\\System32\\config\\SAM",
    "C:\\Windows\\System32\\config\\SECURITY",
];

/// Render untrusted text for a message a terminal will draw.
///
/// A refusal about invisible characters that reprints them is worse than no refusal: U+202E
/// reverses everything after it as it renders, so the message can be made to read as its own
/// opposite, and an ANSI escape recolours or erases the lines around it. Manifests arrive from
/// shared configs, not only from the user's own hand.
///
/// Everything outside printable ASCII-and-ordinary-Unicode is named by codepoint instead of
/// emitted. Ordinary non-ASCII stays as itself — a package name in Cyrillic should read as one
/// — so the rule is drawn at *what the character does to the display*, not at what alphabet it
/// belongs to: C0/C1 controls, the bidi overrides and embeddings, the invisible formatting
/// characters, and the line/paragraph separators.
pub fn printable(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        let dangerous = c.is_control()
            // Bidi overrides, embeddings and isolates — the trojan-source family.
            || ('\u{202A}'..='\u{202E}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c)
            // Zero-width and other invisible formatting.
            || matches!(c, '\u{200B}'..='\u{200F}' | '\u{FEFF}' | '\u{00AD}')
            // Line and paragraph separators, which break a message across lines.
            || matches!(c, '\u{2028}' | '\u{2029}');
        if dangerous {
            out.push_str(&format!("<U+{:04X}>", c as u32));
        } else {
            out.push(c);
        }
    }
    out
}

pub struct Validator;

impl Validator {
    /// Backends whose "name" is legitimately a filesystem path (`link`) or a URL / owner-repo
    /// (`web`, `github`, `appimage`). For these the "looks like an absolute path" guard — a
    /// leading `/` or `\` — would wrongly reject valid input (e.g. `link:/home/me/.vimrc`).
    /// They still get every other check: `..` traversal, the character allowlist, and
    /// shell-injection blocking. Only the leading-separator rule is lifted for them.
    fn is_path_oriented_backend(backend: &str) -> bool {
        // `btrfs` was missing until 2026-07-30, and it is the member whose name is *most*
        // literally a path: `btrfs:/mnt/data/vol` installs by running
        // `btrfs subvolume create /mnt/data/vol`. No declaration of it could be written, and
        // nothing noticed because no harness had a btrfs filesystem to install into.
        //
        // Not `lvm` or `zfs`: `lvm:vg0/data` and `zfs:tank/data` carry a separator and never a
        // leading one, so the strict rule is correct for them and widening it would buy nothing.
        matches!(backend, "link" | "web" | "github" | "appimage" | "btrfs")
    }

    /// Backends whose manager's own identifiers carry a path separator or braces.
    ///
    /// `winget` is the whole list, and a list rather than a rule because those characters are
    /// worth refusing everywhere else: no second manager on any platform names things this way.
    /// `..` stays forbidden for it, exactly as for everything else.
    fn names_carry_windows_identifiers(backend: &str) -> bool {
        backend == "winget"
    }

    /// Refuse a string that reaches a manager's argv *before* it is known to be a package name.
    ///
    /// Two strings do that, and both were live. A **bare** (unprefixed) name is handed to each
    /// candidate manager to find out which one owns it — that probe runs before the model is
    /// collected and therefore before [`Validator::validate_package_name_for`] ever sees the
    /// name. A **search query** is free text by definition and passes no name rule at all. On
    /// Windows both reach managers that ship as `.cmd` shims: `shall search 'q&calc.exe&rem'`
    /// launched `calc.exe`, and one crafted bare name in a shared module wrote files on every
    /// machine that evaluated it.
    ///
    /// **This is the second layer, not the fix.** The fix is that a `.cmd` shim is spawned as
    /// itself so the standard library escapes its arguments (`core/executor.rs`). A comment
    /// calling this validator the thing that stands between a crafted name and a command line
    /// is what let the first layer stay broken.
    pub fn refuse_command_metacharacters(text: &str, what: &str) -> Result<()> {
        if let Some(m) = COMMAND_METACHARACTER_REGEX.find(text) {
            return Err(Error::Validation(format!(
                "`{}` is not usable as {}: it contains `{}`, which a command interpreter reads \
                 as the end of one command and the start of another.",
                printable(text),
                what,
                printable(m.as_str())
            )));
        }
        Ok(())
    }

    /// Validates package names against injection and traversal, with no knowledge of the
    /// backend — the strict rule (a leading path separator is rejected). Prefer
    /// [`Validator::validate_package_name_for`] when the backend is known.
    pub fn validate_package_name(name: &str) -> Result<()> {
        Self::validate_package_name_for(name, "")
    }

    /// Validates a package name for a specific backend. Identical to
    /// [`Validator::validate_package_name`] except that the "absolute path" guard (a leading
    /// `/` or `\`) is lifted for the path/URL-oriented backends (see
    /// [`Validator::is_path_oriented_backend`]), whose names are legitimately paths/URLs.
    /// Directory traversal (`..`), the character allowlist, and shell-injection blocking
    /// always apply, for every backend.
    pub fn validate_package_name_for(name: &str, backend: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Error::Validation("Empty package name".into()));
        }
        if name.len() > 256 {
            return Err(Error::Validation("Name too long".into()));
        }

        // Directory traversal is ALWAYS forbidden, regardless of backend.
        if name.contains("..") {
            return Err(Error::Validation(format!(
                "Path traversal detected in name: {}",
                printable(name)
            )));
        }
        // **A name that opens with `-` is a flag, and no manager has a package called one.**
        //
        // This is what makes a wrong row in the terminator table exploitable rather than merely
        // wrong: `composer:--version` passed every check here and reached composer's argv, on a
        // manager Shall believed `--` would protect and which ignores it (B5). The terminator
        // table stays — it is the mechanism — but it is one boolean per binary measured on one
        // image, and a guard that depends on fifty of those being right is a guard with fifty
        // ways to be wrong.
        //
        // Refused for every backend, including the path-oriented ones: a path may begin with
        // `/` or `~`, and a leading `-` is a flag there too (`rm -rf` reads `-rf` no differently
        // for having come out of a `link:` line).
        if name.starts_with('-') {
            return Err(Error::Validation(format!(
                "`{}` starts with `-`, so a package manager reads it as an option rather than \
                 as a name. No manager has a package called that.",
                printable(name)
            )));
        }

        // A leading path separator normally signals an absolute-path injection attempt — but
        // for a path/URL-oriented backend (e.g. `link`, whose name IS a path) it is valid.
        if !Self::is_path_oriented_backend(backend)
            && (name.starts_with('/') || name.starts_with('\\'))
        {
            return Err(Error::Validation(format!(
                "Path traversal detected in name: {}",
                printable(name)
            )));
        }

        // A manager that prints a name must be able to be handed it back (V.113). `winget`'s
        // identifiers carry backslashes and braces; the grammar was taught to accept them and
        // this check was not, so `adopt` wrote rows that then failed to parse and wedged the
        // model — measured on the native sweep at `adopted.txt:78`.
        let (allowed, injection) = if Self::names_carry_windows_identifiers(backend) {
            (
                &*WINDOWS_IDENTIFIER_NAME_REGEX,
                &*SHELL_INJECTION_REGEX_WINDOWS_ID,
            )
        } else {
            (&*PACKAGE_NAME_REGEX, &*SHELL_INJECTION_REGEX)
        };

        if !allowed.is_match(name) {
            return Err(Error::Validation(format!(
                "Invalid characters in package name: {}",
                printable(name)
            )));
        }

        if injection.is_match(name) {
            return Err(Error::Validation(
                "Shell injection characters detected".into(),
            ));
        }

        Ok(())
    }

    /// Forbidden zones are matched as path prefixes on the *resolved* path, never as
    /// substrings: a substring test both misses `/etc/../etc/shadow` and rejects innocent
    /// names that merely contain a forbidden one. A path that does not exist is returned
    /// unresolved and unchecked — callers must not treat this as proof it is allowed.
    pub async fn validate_path(path: &Path) -> Result<PathBuf> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || Self::validate_path_blocking(&path_owned))
            .await
            .map_err(|e| Error::Other(e.to_string()))?
    }

    /// [`Validator::validate_path`] for a caller that cannot await — the `vars` standard
    /// library's `read_file`, which runs inside Rhai.
    pub fn validate_path_sync(path: &Path) -> Result<PathBuf> {
        Self::validate_path_blocking(path)
    }

    /// Resolve the existing portion of a path even when its final component does not exist.
    /// This preserves the unresolved suffix for callers that want to create/read it, while
    /// still refusing a future path nested below a forbidden directory.
    fn validate_path_blocking(path: &Path) -> Result<PathBuf> {
        for candidate in resolved_prefixes(path)? {
            Self::refuse_forbidden(&candidate)?;
        }
        resolved_prefixes(path)?
            .pop()
            .ok_or_else(|| Error::Validation(format!("cannot resolve {}", path.display())))
    }

    /// Whether `path` lands inside `root`, **with the symlinks in `path`'s existing prefix
    /// followed**.
    ///
    /// **A lexical comparison cannot answer this, and the gap is a hole in two security gates.**
    /// Both callers here ask whether a path the user wrote is inside a directory Shall has a
    /// rule about, and a path can be written one way and land in another: `/tmp/outside/token`
    /// is outside the config repo as text and inside it on disk the moment `/tmp/outside` is a
    /// symlink into the repo, which is the ordinary case for a `link:` target whose parent
    /// directory somebody made a shortcut. The old guard canonicalized only when the target
    /// *existed*, so the first install — the case where it does not — compared strings and let
    /// a plaintext secret through into a git-tracked tree.
    ///
    /// Resolving the existing prefix and keeping the rest is what closes it, and it is the same
    /// resolution [`Validator::validate_path`] already does for a different question, so there is
    /// one implementation of "where does this path actually land".
    ///
    /// **Fails closed.** An unresolvable side is an error rather than `false`: every caller here
    /// is a guard, and a guard that answers "no" because it could not look is a guard that has
    /// stopped guarding. (The two guards ask opposite questions — is this inside the repo, is
    /// this outside the home — and both treat an error as "refuse".)
    pub fn resolves_within(path: &Path, root: &Path) -> Result<bool> {
        let landed = resolved_prefixes(path)?
            .pop()
            .ok_or_else(|| Error::Validation(format!("cannot resolve {}", path.display())))?;
        let base = resolved_prefixes(root)?
            .pop()
            .ok_or_else(|| Error::Validation(format!("cannot resolve {}", root.display())))?;
        let landed = comparable_path(&landed);
        let base = comparable_path(&base);
        Ok(landed.len() >= base.len() && landed[..base.len()] == base[..])
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn refuse_forbidden(canonical: &Path) -> Result<()> {
        let candidate = comparable_path(canonical);
        for forbidden in FORBIDDEN_PATHS {
            let banned = comparable_path(Path::new(forbidden));
            if candidate.len() >= banned.len() && candidate[..banned.len()] == banned[..] {
                warn!(
                    "Security Block: Attempted access to forbidden path: {:?}",
                    canonical
                );
                return Err(Error::Permission(format!("Access Denied: {}", forbidden)));
            }
        }
        Ok(())
    }
}

/// Every prefix of `path` from the fully resolved existing one down to `path` itself.
///
/// The existing ancestor is canonicalized (which follows the symlinks in it), and each
/// unresolved component is pushed back on in turn, so the caller can ask about *any* of them —
/// which is what a forbidden-path check needs, since a future `sub/shadow` is as forbidden as a
/// present one. The list is ordered outermost first, so the last element is the path itself.
fn resolved_prefixes(path: &Path) -> Result<Vec<PathBuf>> {
    if let Ok(canonical) = path.canonicalize() {
        return Ok(vec![canonical]);
    }

    let mut missing = Vec::new();
    let mut ancestor = path;
    while !ancestor.exists() {
        let Some(parent) = ancestor.parent() else {
            return Ok(vec![path.to_path_buf()]);
        };
        if let Some(name) = ancestor.file_name() {
            missing.push(name.to_os_string());
        }
        ancestor = parent;
    }

    let mut candidate = ancestor
        .canonicalize()
        .map_err(|e| Error::Validation(format!("Path resolution failed: {}", e)))?;
    let mut out = vec![candidate.clone()];
    for component in missing.iter().rev() {
        candidate.push(component);
        out.push(candidate.clone());
    }
    Ok(out)
}

/// A canonical path reduced to comparable segments, with the Windows verbatim marker removed.
///
/// **This is the whole fix.** `canonicalize()` answers `\\?\C:\Windows\…` on Windows, and
/// `Path::starts_with` compares prefix *kinds* before anything else — `VerbatimDisk ≠ Disk`, so
/// every entry in [`FORBIDDEN_PATHS`] missed and the control was compiled dead on the platform
/// whose registry hives it exists for. Segments rather than a joined string, so a prefix match
/// stays a match on whole components: `...\SAM` must not swallow `...\SAMBA`.
///
/// Lowercased on Windows only, because NTFS compares names that way and a caller handing over a
/// differently-cased path must meet the same wall.
fn comparable_path(path: &Path) -> Vec<String> {
    let text = path.as_os_str().to_string_lossy();
    let text = match text.strip_prefix(r"\\?\UNC\") {
        Some(unc) => format!(r"\\{unc}"),
        None => text
            .strip_prefix(r"\\?\")
            .map(str::to_string)
            .unwrap_or_else(|| text.into_owned()),
    };
    let lower = cfg!(windows);
    text.split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .map(|s| {
            if lower {
                s.to_ascii_lowercase()
            } else {
                s.to_string()
            }
        })
        .collect()
}

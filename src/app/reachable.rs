//! A package that installed and cannot be run.
//!
//! `shall install pub:sass` succeeds, `shall list` agrees, and typing `sass` answers "command
//! not found" — because `~/.pub-cache/bin` is not on `PATH` and nothing ever said so. Shall
//! reported success for a package the user cannot invoke, which is the same event as a failed
//! install everywhere it matters and is reported as the opposite.
//!
//! Every per-user ecosystem has this shape: the manager installs into a directory under `$HOME`
//! and leaves putting it on `PATH` to you. System managers do not — `apt` writes to `/usr/bin`,
//! which is on every `PATH` by definition — so this says nothing about them.
//!
//! One place and one check on the shared path, not a check per backend. Eleven copies of a
//! `~/.x/bin` string is eleven chances to disagree, and the eleventh is the one that stays
//! wrong.
//!
//! **A convention is only usable where there is one.** The first version of this file cleared
//! `npm`/`yarn`/`pnpm`/`bun` as "they shim into a directory their own installer wires up", and
//! CI falsified that for yarn on 2026-07-29 — an install that passed `list` and left the binary
//! unreachable. Those managers are asked instead, because the answer is decided by how the
//! manager itself was installed and no constant is right on two machines.

use crate::config::Config;
use crate::core::CommandExecutor;
use std::path::{Path, PathBuf};

/// Every directory that would answer "where did that executable go", most authoritative first.
///
/// Empty means "not this manager's problem": either it installs into a system directory, or it
/// installs no executables at all.
///
/// Three kinds of answer, and the difference matters:
///
/// - **Shall's own deploy directory** for the artifact backends. It is `[bin_dir]` and nothing
///   else, so the sentence this file prints cannot name a directory the deploy did not use.
/// - **The tool's own answer**, for the managers whose global directory is decided by how they
///   were installed, or by which switch is selected. `yarn global bin` reads `…\scoop\apps\yarn\current\global\bin` on the
///   developer's box and `%LOCALAPPDATA%\Yarn\bin` on a clean runner; a constant would be wrong
///   on one of them, and was.
/// - **The ecosystem's convention**, for the managers that have one, with the environment
///   variable first: a user who set `GOBIN` or `GEM_HOME` has already answered this question and
///   a hard-coded `~/go/bin` would contradict them.
///
/// More than one entry means the answer is genuinely ambiguous (npm's prefix), and any one of
/// them being on `PATH` is enough to stay quiet — a warning that fires on a machine that is
/// fine is how the real one gets ignored.
pub async fn user_bin_dirs(backend: &str, cfg: &Config, exec: &CommandExecutor) -> Vec<PathBuf> {
    if deploys_through_shall(backend) {
        return vec![cfg.bin_dir.clone()];
    }
    if let Some(dirs) = asks_the_tool(backend, exec).await {
        return dirs;
    }
    if backend == "go" {
        // One implementation of "where does `go install` put it", shared with the backend that
        // has to find the binary afterwards. Two readings of `GOPATH` is two answers.
        return crate::backends::go::install_bin_dir(exec)
            .await
            .into_iter()
            .collect();
    }
    conventional_bin_dir(backend).into_iter().collect()
}

/// The backends whose executables Shall itself deploys, into `[bin_dir]`.
fn deploys_through_shall(backend: &str) -> bool {
    matches!(backend, "github" | "web" | "appimage" | "shim")
}

/// The managers that know where their own global directory is, and are the only ones who do.
///
/// A tool that cannot answer — not installed, or a version without the subcommand (yarn berry
/// dropped `global`) — produces no candidates and therefore no warning. Claiming a directory
/// nobody confirmed is how a warning starts being wrong.
async fn asks_the_tool(backend: &str, exec: &CommandExecutor) -> Option<Vec<PathBuf>> {
    let (prog, args): (&str, &[&str]) = match backend {
        "yarn" => ("yarn", &["global", "bin"]),
        "pnpm" => ("pnpm", &["bin", "-g"]),
        "bun" => ("bun", &["pm", "bin", "-g"]),
        "npm" => ("npm", &["prefix", "-g"]),
        // opam's directory belongs to the current switch, so there is no constant to write:
        // `/root/.opam/default/bin` here, something else on a machine with another switch
        // selected, and nothing at all without one. Measured in the `tools` image 2026-07-29,
        // where `opam install -y ocamlfind` succeeded and `ocamlfind` was on nobody's PATH —
        // opam expects `eval $(opam env)`, which a declaration cannot do for you.
        "opam" => ("opam", &["var", "bin"]),
        _ => return None,
    };
    let out = exec.run_output(prog, args, false).await.ok()?;
    let dir = first_absolute_path(&out)?;
    Some(npm_prefix_candidates(backend, dir))
}

/// `npm prefix -g` prints the prefix, not the bin directory: executables land in `<prefix>`
/// on Windows and `<prefix>/bin` everywhere else. Both are candidates, so the platform rule
/// being wrong somewhere cannot produce a warning about a machine that is fine.
pub fn npm_prefix_candidates(backend: &str, dir: PathBuf) -> Vec<PathBuf> {
    if backend != "npm" {
        return vec![dir];
    }
    if cfg!(windows) {
        vec![dir.clone(), dir.join("bin")]
    } else {
        vec![dir.join("bin"), dir]
    }
}

/// The first line of a tool's output that is an absolute path.
///
/// Line-by-line rather than "trim the whole thing", because `yarn` prefixes its answer with
/// `warning …` lines often enough that reading the first line would report a warning as a
/// directory.
pub fn first_absolute_path(out: &str) -> Option<PathBuf> {
    out.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .find(|p| p.is_absolute())
}

/// Where a manager puts the executables it installs, for the managers whose answer is a
/// convention rather than a question you can ask.
pub fn conventional_bin_dir(backend: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let env_path = |k: &str| {
        std::env::var(k)
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };

    Some(match backend {
        "cargo" => env_path("CARGO_INSTALL_ROOT")
            .or_else(|| env_path("CARGO_HOME"))
            .map(|p| p.join("bin"))
            .unwrap_or_else(|| home.join(".cargo").join("bin")),
        "gem" => env_path("GEM_HOME")
            .map(|p| p.join("bin"))
            .unwrap_or_else(|| {
                home.join(".local")
                    .join("share")
                    .join("gem")
                    .join("ruby")
                    .join("bin")
            }),
        "pub" => env_path("PUB_CACHE")
            .map(|p| p.join("bin"))
            .unwrap_or_else(|| home.join(".pub-cache").join("bin")),
        "nimble" => home.join(".nimble").join("bin"),
        "luarocks" => home.join(".luarocks").join("bin"),
        "mix" => home.join(".mix").join("escripts"),
        "stack" => home.join(".local").join("bin"),
        "krew" => home.join(".krew").join("bin"),
        "pipx" => env_path("PIPX_BIN_DIR").unwrap_or_else(|| home.join(".local").join("bin")),
        "composer" => home
            .join(".config")
            .join("composer")
            .join("vendor")
            .join("bin"),
        _ => return None,
    })
}

/// Is `dir` one of the entries in `PATH`?
///
/// Compared after `canonicalize`, so `~/go/bin` and `/home/u/go/bin/` and a `PATH` entry
/// reached through a symlink are one directory rather than three. A path that cannot be
/// canonicalised has not been created yet, and a directory that does not exist is not on
/// anyone's `PATH` — but the raw comparison still runs first, because a user may well have
/// added the entry before the manager created the directory.
pub fn is_on_path(dir: &Path) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let canonical = dir.canonicalize().ok();
    std::env::split_paths(&path)
        .any(|p| p == dir || (canonical.is_some() && p.canonicalize().ok() == canonical))
}

/// The command that puts `dir` on `PATH`, in the form this platform actually takes.
///
/// `export PATH=…` on Windows is not a smaller fix, it is a wrong one: it names a shell the
/// user is not in and a syntax cmd and PowerShell both reject. A line someone has to translate
/// before they can run it is the "add it to your PATH" advice this warning exists to replace.
fn how_to_add(dir: &Path) -> String {
    if cfg!(windows) {
        // `setx` expands and rewrites PATH through a legacy 1024-character buffer. Use a
        // PowerShell user-scope update instead; it reads the current value and writes the full
        // string without truncating long developer or CI PATHs.
        let path = dir
            .display()
            .to_string()
            .replace('`', "``")
            .replace('"', "`\"");
        format!(
            "powershell -NoProfile -Command \"[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')\"",
            path
        )
    } else {
        format!("export PATH=\"{}:$PATH\"", dir.display())
    }
}

/// The sentence a user can act on, or `None` when there is nothing to say.
///
/// Names the directory and the exact command, because "add it to your PATH" is advice and a
/// line you can paste is a fix. A warning, never a refusal: the package really did install,
/// and the machine really is closer to the files than it was.
pub async fn unreachable_warning(
    backend: &str,
    cfg: &Config,
    exec: &CommandExecutor,
) -> Option<String> {
    let dirs = user_bin_dirs(backend, cfg, exec).await;
    if dirs.iter().any(|d| is_on_path(d)) {
        return None;
    }
    let dir = dirs.into_iter().next()?;
    Some(format!(
        "`{backend}` installs its executables into {}, which is not on your PATH — so what it \
         just installed will answer \"command not found\".\n  Put it on your PATH with:\n    {}",
        dir.display(),
        how_to_add(&dir)
    ))
}

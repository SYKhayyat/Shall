//! How this platform runs a script file.
//!
//! One answer, because there are three callers — an `exec:` declaration (XIII.3), a hook on one
//! of Shall's own events (XIII.13), and a `#!` package hook (V.150) — and a second copy would be
//! a second chance for one of them to be wrong on Windows, where the answer is not "make it
//! executable and run it". A fourth asks only half the question: a `vars.<ext>` provider names
//! its interpreter by file extension (IX.6) and takes [`interpreter_named`] to find it.
//!
//! **The `#!` line is read here, not left to the kernel.** Windows has no shebang mechanism, so
//! a script handed to `CreateProcess` fails with *"not a valid application for this OS
//! platform"* whatever its first line says — naming the interpreter ourselves is the only way
//! one hook runs on both platforms. On Unix this resolves to the binary the kernel would have
//! launched anyway, so the platforms agree rather than diverge.
//!
//! Not pure: finding the interpreter is a PATH lookup, and it goes through the process-wide memo
//! in `core::executor` so this is not a second answer to "where does a program live".

use crate::core::{Error, Result};
use std::path::Path;

/// The extension a script file must carry when *this platform's default* interpreter runs it.
///
/// **Paired with the interpreter deliberately.** `powershell -File` refuses a file whose name
/// does not end in `.ps1` — so a caller that writes a script to a temporary file and picks the
/// interpreter from here must take the suffix from here too, or it gets a refusal about the
/// file extension on Windows and nothing at all on Linux.
pub const SCRIPT_SUFFIX: &str = if cfg!(windows) { ".ps1" } else { ".sh" };

/// A program to run, with the script already in place as its last argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
}

/// What a script's first line asks to be run with, before anything is looked up on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shebang {
    /// The interpreter exactly as written — `/usr/bin/python3`, `python3`, `/bin/bash`.
    pub interpreter: String,
    /// Whatever followed it on the line.
    pub args: Vec<String>,
}

/// The command that runs the script at `path`, whose text is `contents`.
///
/// A script with no `#!` gets this platform's default shell, which is what every script in a
/// Shall config was before shebangs were honoured anywhere.
pub fn launch_for(path: &Path, contents: &str) -> Result<Launch> {
    let script = path.to_string_lossy().to_string();

    let (program, args) = match shebang_of(contents)? {
        None => platform_default(script),
        Some(shebang) => {
            let program =
                interpreter_named(&shebang.interpreter).ok_or_else(|| unavailable(&shebang))?;
            let mut args = shebang.args;
            // PowerShell reads a bare positional argument as a `-Command` expression, not as a
            // file to run, so a shebang naming it needs the flag the default form already passes.
            let launcher = program_name(&program);
            if launcher.eq_ignore_ascii_case("powershell") || launcher.eq_ignore_ascii_case("pwsh")
            {
                args.push("-File".to_string());
            }
            args.push(script);
            (program, args)
        }
    };

    // An interpreter on Windows can itself be a `.cmd`/`.bat`/`.ps1` shim, which `CreateProcess`
    // cannot launch at all. Everything else Shall runs goes through this; an interpreter chosen
    // here is not the one thing exempt from it.
    let (program, args) = crate::core::launch::effective_command(&program, &args);
    Ok(Launch { program, args })
}

/// The interpreter a script's first line asks for, if it asks for one.
///
/// Leading whitespace is allowed before the `#!` because a hook written in a TOML multi-line
/// string arrives indented and with a leading newline, which is how the shipped example config
/// writes them. A Unix kernel would reject that file — it wants `#!` at byte zero — but the
/// interpreter is named here rather than by the kernel, so the restriction no longer applies and
/// the same text now runs on both platforms.
pub fn shebang_of(contents: &str) -> Result<Option<Shebang>> {
    let Some(line) = contents.trim_start().lines().next() else {
        return Ok(None);
    };
    let Some(rest) = line.strip_prefix("#!") else {
        return Ok(None);
    };

    let mut tokens = rest.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Ok(None);
    }

    // `env` is a PATH search wearing a path. The search happens in `interpreter_named`, so the
    // indirection is removed here rather than launched — there is no `/usr/bin/env` on Windows.
    if program_name(tokens[0]) == "env" {
        tokens.remove(0);
        if tokens.first() == Some(&"-S") {
            tokens.remove(0);
        }
        // `env` can set variables before the interpreter runs. Shall cannot: one of the three
        // callers runs its script through an executor with no per-command environment, and a
        // form honoured by two callers out of three is worse than one refused by all three.
        if let Some(assignment) = tokens.first().copied().filter(|t| is_assignment(t)) {
            return Err(Error::Other(format!(
                "this script's first line sets `{}` before its interpreter, which Shall does not \
                 do. Set it inside the script instead.",
                assignment
            )));
        }
    }

    let Some((interpreter, args)) = tokens.split_first() else {
        return Ok(None);
    };
    Ok(Some(Shebang {
        interpreter: (*interpreter).to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
    }))
}

/// This platform's shell, for a script that named no interpreter of its own.
fn platform_default(script: String) -> (String, Vec<String>) {
    if cfg!(windows) {
        (
            "powershell".to_string(),
            vec![
                // No profile: a user's `$PROFILE` is not part of the script they wrote, and a
                // hook that behaves differently on one machine because of it is undebuggable.
                "-NoProfile".to_string(),
                // The script is already gated by II.12's approval ledger, which is a stronger
                // check than the execution policy and the reason this is not a hole.
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-File".to_string(),
                script,
            ],
        )
    } else {
        ("sh".to_string(), vec![script])
    }
}

/// Where this machine keeps an interpreter a config named.
///
/// An absolute path that exists is taken as written — on Unix that is the binary the kernel
/// itself would have launched, so honouring the shebang here changes nothing there. Everything
/// else is looked up by name, because `/bin/bash` and `/usr/bin/python3` are Unix spellings of
/// programs that live somewhere else entirely on Windows, and the name is the only part of such
/// a line that travels between machines.
///
/// Public because a `#!` line is not the only way a config names an interpreter: a `vars.py`
/// provider names one by its extension (IX.6). Those are one question, and answering it twice is
/// how `.py` came to mean exactly `python` on Windows — no `python3`, no `py` — while a `#!` line
/// got all three.
pub fn interpreter_named(interpreter: &str) -> Option<String> {
    let as_written = Path::new(interpreter);
    if as_written.is_absolute() && as_written.is_file() {
        return Some(interpreter.to_string());
    }
    let name = program_name(interpreter);
    candidates(name)
        .iter()
        .find_map(|candidate| crate::core::launch::resolve_program(candidate))
        .map(|found| found.to_string_lossy().into_owned())
}

/// The names to try for an interpreter, in order: the one that was written, then the spellings
/// this platform uses for the same program.
///
/// The alternates are Windows-only and short on purpose — this is "the same program under the
/// name this OS gives it", not a search for something similar. Python is the case that matters:
/// a shebang says `python3` because that is what Unix calls it, and a Windows install is almost
/// always `python`, with `py` (the launcher every python.org installer puts on PATH) there when
/// neither name is.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn candidates(name: &str) -> Vec<&str> {
    #[cfg(windows)]
    {
        let alternates: &[&str] = match name.to_ascii_lowercase().as_str() {
            "python3" => &["python", "py"],
            "python" => &["py"],
            "sh" => &["bash"],
            "pwsh" => &["powershell"],
            "node" => &["nodejs"],
            _ => &[],
        };
        let mut out = vec![name];
        out.extend_from_slice(alternates);
        out
    }
    #[cfg(not(windows))]
    {
        vec![name]
    }
}

/// The last path segment of an interpreter, with a Windows `.exe` removed.
///
/// Backslashes split too. Shebangs are a Unix form and are written with forward slashes, but a
/// Windows author editing a config on their own machine will eventually write the other one, and
/// treating `C:\Python\python.exe` as a single nameless segment helps nobody.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn program_name(interpreter: &str) -> &str {
    let name = interpreter
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(interpreter);
    match name.len().checked_sub(4) {
        // `cut > 0`: a file actually called `.exe` has no name left once the suffix is taken off
        // it, and an empty interpreter resolves to whatever `which("")` feels like answering.
        Some(cut)
            if cut > 0
                && name.is_char_boundary(cut)
                && name[cut..].eq_ignore_ascii_case(".exe") =>
        {
            &name[..cut]
        }
        _ => name,
    }
}

/// `NAME=VALUE`, as `env` accepts it — not a path that happens to contain an `=`.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn is_assignment(token: &str) -> bool {
    match token.split_once('=') {
        Some((name, _)) => {
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.starts_with(|c: char| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Why the script cannot run here, in terms of the line the author wrote and what was looked for.
///
/// Names every spelling that was tried: the alternates exist precisely so that a `python3`
/// shebang finds a Windows `python`, and a reader who is not told they were tried will go
/// installing a second Python under a third name.
fn unavailable(shebang: &Shebang) -> Error {
    let name = program_name(&shebang.interpreter);
    let tried = candidates(name).join(", ");
    Error::Other(format!(
        "this script's first line asks for `{}`, and this machine has no `{}` on PATH \
         (tried: {}). Install it, or name an interpreter this machine has.",
        shebang.interpreter, name, tried
    ))
}

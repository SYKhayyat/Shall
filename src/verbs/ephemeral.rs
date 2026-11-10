//! **The two verbs that run something without declaring it.**
//!
//! `shell` and `run` are deliberately outside the model (II.8): a shell session is not a
//! declaration, and writing a file for something that ends when the shell does would leave the
//! file behind. That shared property is why they are one module — it is the thing a reader has
//! to know before touching either.

use crate::verbs::prelude::*;

pub async fn handle_shell(shell: crate::app::EphemeralShell, packages: &[String]) -> Result<()> {
    shell.enter(packages).await.map_err(|e| e.into())
}

/// `shall run --packages X -- cmd arg…`
///
/// **One rule, both spellings.** The first positional may still carry a whole command line, which
/// is what the quoted form (`-- "jq -r .name"`) has always meant; everything after it is an
/// argument, verbatim. The second half is new: `command` was a lone positional, so clap refused
/// `-- jq -r .name` outright — and with it `src/bin/shim.rs`, which builds exactly that argv and
/// is the entire mechanism behind a `@shim=true` line.
pub async fn handle_run(
    runner: crate::app::Runner,
    packages: &[String],
    command: &str,
    trailing: &[std::ffi::OsString],
) -> Result<()> {
    let mut parts: Vec<std::ffi::OsString> = command.split_whitespace().map(Into::into).collect();
    parts.extend(trailing.iter().cloned());
    let Some((bin, args)) = parts.split_first() else {
        return Err(
            crate::core::Error::Validation("`shall run` needs a command to run".into()).into(),
        );
    };
    let bin = bin.to_string_lossy();
    runner
        .run(packages, bin.as_ref(), args)
        .await
        .map_err(|e| e.into())
}

//! `shall path` and `shall edit` — finding your files.
//!
//! Without these, every user memorises `~/.config/shall` and every script hard-codes it,
//! which is how a configurable path stops being configurable in practice.

use crate::config::settings::{resolve_root, ResolvedRoot, RootSource, Settings};
use crate::core::{Error, Result};
use crate::model::Layout;
use std::path::{Path, PathBuf};

/// Where the repo is, and which of the four sources said so.
pub fn locate(flag: Option<&Path>) -> Result<ResolvedRoot> {
    resolve_root(flag, &Settings::load()?)
}

/// Plain output is exactly one line — the directory — so `cd $(shall path)` works. Anything
/// explanatory goes behind `--explain`, on stdout only when asked for.
pub fn render_path(resolved: &ResolvedRoot, explain: bool) -> String {
    let mut out = resolved.path.display().to_string();
    if explain {
        out.push_str(&format!("\n\nset by: {}", resolved.source.describe()));
        out.push_str(&format!("\nsettings file: {}", Settings::path().display()));
        if resolved.source != RootSource::SettingsFile && !Settings::path().exists() {
            out.push_str("\n  (no settings file yet — `shall path --set DIR` writes one)");
        }
        if !resolved.path.exists() {
            out.push_str("\n\nThis directory does not exist yet. `shall init` creates it.");
        }
    }
    out
}

/// Store the repo location so later runs need no flag and no environment variable.
///
/// The settings file, and whether it was written — a preview names it and leaves it alone.
pub fn set_root(dir: &Path) -> Result<(PathBuf, bool)> {
    let dir = crate::config::settings::absolute_or_refuse(dir.to_path_buf(), "`shall path --set`")?;
    let mut settings = Settings::load()?;
    settings.config_root = Some(dir);
    let stored = settings.save()?;
    Ok((Settings::path(), stored))
}

/// A file the user named on `shall edit`, resolved inside the repo.
///
/// Refuses anything that climbs out. `shall edit ../../.bashrc` would otherwise make this
/// command an arbitrary-file editor that happens to live under a package manager.
pub fn resolve_target(root: &Path, file: Option<&str>) -> Result<PathBuf> {
    let Some(file) = file else {
        return Ok(root.to_path_buf());
    };

    let normalised = file.replace('\\', "/");
    let relative = PathBuf::from(&normalised);
    // A leading `/` is not `is_absolute()` on Windows — there is no drive letter — but it
    // still escapes the repo, so the textual check has to come with the type's own.
    if relative.is_absolute() || normalised.starts_with('/') || has_drive_prefix(&normalised) {
        return Err(Error::Validation(format!(
            "`{}` is an absolute path. `shall edit` opens files inside your config repo; \
             name one relative to it, like `modules/dev.txt`.",
            file
        )));
    }
    for part in relative.components() {
        if matches!(part, std::path::Component::ParentDir) {
            return Err(Error::Validation(format!(
                "`{}` climbs out of your config repo. `shall edit` opens files inside it.",
                file
            )));
        }
    }

    let target = root.join(&relative);
    if target.exists() {
        return Ok(target);
    }

    Err(Error::Validation(format!(
        "`{}` is not in your config repo ({}).\n  It holds: {}",
        file,
        root.display(),
        known_files(root).join(", ")
    )))
}

/// `C:/Users/...` — absolute on Windows, and merely an odd relative name on unix, where
/// joining it to the repo root would silently produce a directory called `C:`.
fn has_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// The II.1 layout, named for someone who mistyped one of them.
fn known_files(root: &Path) -> Vec<String> {
    let layout = Layout::new(root.to_path_buf(), root.to_path_buf());
    [
        layout.modules_dir(),
        layout.profiles_dir(),
        layout.active_file(),
        layout.priority_file(),
        layout.schedules_file(),
    ]
    .iter()
    .filter_map(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .filter(|_| p.exists())
    })
    .collect::<Vec<_>>()
}

pub fn editor_command() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| default_editor().to_string())
}

fn default_editor() -> &'static str {
    if cfg!(windows) {
        "notepad"
    } else {
        "vi"
    }
}

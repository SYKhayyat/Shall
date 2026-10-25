//! Shall's own settings — the one file that is not in your repo.
//!
//! It holds where your repo is, and nothing else. A key inside the repo saying where the repo
//! is would have to be read out of the file whose location it defines, and no ordering
//! resolves that. A key in a fixed location resolves it in one step: Shall reads its own
//! settings from a place it always knows, learns the repo path, and everything after that is
//! the ordinary model.

use crate::core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The only key this file may hold. Enforced by the parser rather than by discipline: a file
/// holding exactly one key is the file that grows a second one, and the moment it does there
/// are two preference systems and a new question about which wins on every key either could
/// hold. The one key it holds is the one key `preferences.toml` structurally cannot.
const ONLY_KEY: &str = "config_root";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_root: Option<PathBuf>,
}

/// Which of the four sources answered "where is the repo". Carried rather than discarded so
/// `shall path` can say why the answer is what it is — a wrong answer is then debuggable in
/// one command instead of by elimination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSource {
    Flag,
    Environment,
    SettingsFile,
    Default,
}

impl RootSource {
    pub fn describe(self) -> &'static str {
        match self {
            RootSource::Flag => "--config-dir",
            RootSource::Environment => "$SHALL_CONFIG_DIR",
            RootSource::SettingsFile => "your Shall settings file",
            RootSource::Default => "the built-in default",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedRoot {
    pub path: PathBuf,
    pub source: RootSource,
}

impl Settings {
    /// The platform config directory, not the data directory: this is configuration, and
    /// putting it beside the data invites the assumption that deleting the data dir is safe.
    ///
    /// A file rather than `shall/settings.toml`, because **the default repo is
    /// `<config dir>/shall`** — so the obvious nested spelling puts this file inside the repo
    /// it exists to locate, where git would commit a machine-specific absolute path and a
    /// fleet would share one box's answer.
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shall.settings.toml")
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path())
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            // No settings file is the normal case, not a failure: the default repo location
            // is what most machines use.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
            Err(e) => return Err(Error::Io(e.to_string())),
        };
        Self::parse(&text, path)
    }

    pub fn parse(text: &str, path: &Path) -> Result<Self> {
        let table: toml::Table = toml::from_str(crate::config::without_bom(text))
            .map_err(|e| Error::Config(format!("{} is not valid TOML: {}", path.display(), e)))?;

        for key in table.keys() {
            if key != ONLY_KEY {
                return Err(Error::Config(format!(
                    "`{}` is not allowed in {}.\n  This file holds `{}` and nothing else — \
                     where your repo is. Everything about how Shall behaves goes in \
                     `preferences.toml`, inside the repo, where it is versioned with the \
                     config it describes.",
                    key,
                    path.display(),
                    ONLY_KEY
                )));
            }
        }

        let config_root = match table.get(ONLY_KEY) {
            None => None,
            Some(v) => {
                let raw = v.as_str().ok_or_else(|| {
                    Error::Config(format!(
                        "`{}` in {} must be a path in quotes.",
                        ONLY_KEY,
                        path.display()
                    ))
                })?;
                Some(absolute_or_refuse(
                    PathBuf::from(raw),
                    &format!("`{}` in {}", ONLY_KEY, path.display()),
                )?)
            }
        };

        Ok(Settings { config_root })
    }

    /// Returns whether the bytes reached the disk; a preview writes none.
    pub fn save(&self) -> Result<bool> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Toml(e.to_string()))?;
        crate::utils::file::persist(&Self::path(), &text)
    }
}

/// The one refusal, so every door that names a directory says the same sentence.
///
/// It said it in one place — `shall path --set` — and the other three doors said nothing, which
/// is how `--config-dir ./sandbox` came to read `preferences.toml` from the sandbox and
/// `modules/` from the real repo (AU2). `source` names the door, because "which of these four
/// did I get wrong" is the next question after "one of them is relative".
pub fn absolute_or_refuse(path: PathBuf, source: &str) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(Error::Config(format!(
            "{} is `{}`, which is not an absolute path.\n  A relative path would mean a \
             different directory depending on where you ran Shall from.",
            source,
            path.display()
        )));
    }
    Ok(path)
}

/// Command line beats environment beats stored beats default.
///
/// `flag` is `--config-dir`. The environment variable is read here rather than by the caller
/// so that every path answering "where is the repo" answers it the same way.
///
/// **Fallible, because a relative answer is refused rather than replaced.** Discarding it and
/// carrying on is what inverted the documented precedence: the flag was dropped downstream and
/// `$SHALL_CONFIG_DIR` — which `--help` says the flag outranks — was picked back up in its
/// place, silently, by the fallback.
pub fn resolve_root(flag: Option<&Path>, settings: &Settings) -> Result<ResolvedRoot> {
    if let Some(path) = flag {
        return Ok(ResolvedRoot {
            path: absolute_or_refuse(path.to_path_buf(), "`--config-dir`")?,
            source: RootSource::Flag,
        });
    }
    if let Some(dir) = std::env::var_os("SHALL_CONFIG_DIR").filter(|v| !v.is_empty()) {
        return Ok(ResolvedRoot {
            path: absolute_or_refuse(PathBuf::from(dir), "`$SHALL_CONFIG_DIR`")?,
            source: RootSource::Environment,
        });
    }
    if let Some(stored) = &settings.config_root {
        return Ok(ResolvedRoot {
            path: stored.clone(),
            source: RootSource::SettingsFile,
        });
    }
    Ok(ResolvedRoot {
        path: crate::utils::safe_config_dir(),
        source: RootSource::Default,
    })
}

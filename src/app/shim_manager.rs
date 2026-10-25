use crate::core::account::Account;
use crate::core::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

/// Bytes present in every Shall binary, past and future, precisely so a deployed copy can be
/// recognised as Shall's whatever version it is.
///
/// Referenced through `std::hint::black_box` in [`bytes_contain_marker`] — the reference is
/// what keeps the constant in the binary's read-only data across builds and optimizations.
pub const SHIM_MARKER: &[u8] = b"SHALL::DEPLOYED-SHIM::9f2c41e7::v1";

/// Whether the file at `path` carries [`SHIM_MARKER`].
async fn bytes_contain_marker(path: &Path) -> bool {
    // The black box is load-bearing for future binaries; today's binary also contains the
    // marker because this very function names it.
    std::hint::black_box(SHIM_MARKER);
    match fs::read(path).await {
        Ok(bytes) => {
            bytes.len() >= SHIM_MARKER.len()
                && bytes.windows(SHIM_MARKER.len()).any(|w| w == SHIM_MARKER)
        }
        Err(_) => false,
    }
}

/// A shim is the shall binary itself, deployed under the target's name: on startup shall
/// reads `current_exe()`'s filename and re-dispatches when it is not its own
/// (`attempt_shim_hijack`). The shim's NAME is therefore the entire mechanism.
pub struct ShimManager {
    bin_dir: PathBuf,
    account: Option<Account>,
}

pub fn shim_target(
    config_bin_dir: &Path,
    user: Option<&str>,
) -> Result<(PathBuf, Option<Account>)> {
    match user {
        None => Ok((config_bin_dir.to_path_buf(), None)),
        Some(name) => {
            let account = Account::resolve(name)?;
            Ok((account.bin_dir()?, Some(account)))
        }
    }
}

impl ShimManager {
    /// The directory comes from `Config::bin_dir` and from nowhere else. A constructor that
    /// resolved `~/.local/bin` itself is a second answer to "where do shims go", and it is
    /// the answer a sandbox cannot move.
    pub async fn with_bin_dir(bin_dir: PathBuf) -> Result<Self> {
        Self::build(bin_dir, None).await
    }

    pub async fn for_user(config_bin_dir: &Path, user: Option<&str>) -> Result<Self> {
        let (bin_dir, account) = shim_target(config_bin_dir, user)?;
        Self::build(bin_dir, account).await
    }

    async fn build(bin_dir: PathBuf, account: Option<Account>) -> Result<Self> {
        if !tokio::fs::try_exists(&bin_dir).await.unwrap_or(false) {
            debug!("Creating shim directory at {:?}", bin_dir);
            crate::utils::file::ensure_dir_async(&bin_dir).await?;
        }

        Ok(Self { bin_dir, account })
    }

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Whether `path` is a shim Shall deployed, i.e. the shall binary under another name.
    ///
    /// **Identified by a marker baked into the binary, not by byte-equality with the running
    /// exe.** Byte-equality answered "is this THE CURRENT shall?" — so after any self-upgrade,
    /// every existing shim stopped matching, `real_program` resolved names TO the stale shim,
    /// skipped it, fell back to the bare name, and the OS resolved that back to the shim: an
    /// unbounded shall-spawning chain whenever the bin dir was on PATH. The marker is stable
    /// across versions, which is the property "was this deployed by Shall?" needs.
    ///
    /// `bin_dir` is `~/.local/bin`, which Shall shares with the user and with every other
    /// tool that installs there. Without this test, removal deletes by NAME alone, so a
    /// managed package called `jq` makes every sync delete whatever `~/.local/bin/jq` is —
    /// a file Shall never created and does not own.
    /// `pub(crate)` for one caller beyond this file: the runner has to know a shim when it sees
    /// one on `PATH`, because running a shim is how a shim re-enters Shall for ever.
    pub(crate) async fn is_deployed_shim(path: &Path) -> bool {
        if bytes_contain_marker(path).await {
            return true;
        }
        let Ok(current_exe) = std::env::current_exe() else {
            return false;
        };
        let (Ok(shim_meta), Ok(exe_meta)) = (
            tokio::fs::symlink_metadata(path).await,
            tokio::fs::metadata(&current_exe).await,
        ) else {
            return false;
        };
        // A shim is a hard link or a copy — never a symlink (create_shim documents why).
        if shim_meta.file_type().is_symlink() || shim_meta.len() != exe_meta.len() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if shim_meta.dev() == exe_meta.dev() && shim_meta.ino() == exe_meta.ino() {
                return true;
            }
        }
        // Same size but not the same inode: either the copy fallback, or an unrelated file
        // that happens to match. Only the bytes can tell them apart.
        match (
            tokio::fs::read(path).await,
            tokio::fs::read(&current_exe).await,
        ) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    /// Where a shim for this name lives. One definition, because a second copy of the `.exe`
    /// rule is a second copy that can disagree — and the caller that asks "is this shim in
    /// effect?" has to look at exactly the path `create_shim` writes.
    pub fn shim_path(&self, binary_name: &str) -> PathBuf {
        #[allow(unused_mut)] // mutated only under cfg(windows)
        let mut path = self.bin_dir.join(binary_name);
        #[cfg(windows)]
        {
            if path.extension().is_none_or(|ext| ext != "exe") {
                path.set_extension("exe");
            }
        }
        path
    }

    /// Whether a `shim:` declaration is in effect on this machine right now.
    ///
    /// Asks the disk, not the ledger. A shim recorded as applied and then deleted by hand is
    /// exactly the drift `check` is for, and the record cannot see it.
    pub async fn is_in_effect(&self, binary_name: &str) -> bool {
        // Shall itself is its own shim: `create_shim("shall")` correctly does nothing, and
        // without this arm the drift report nagged forever about a file that must not exist.
        if binary_name == "shall" {
            return true;
        }
        Self::is_deployed_shim(&self.shim_path(binary_name)).await
    }

    pub async fn create_shim(&self, binary_name: &str) -> Result<()> {
        let target_path = self.shim_path(binary_name);

        // A "shall" shim would overwrite shall itself with itself — and on the copy path,
        // truncate the running binary. `is_in_effect` answers true for this name, so the
        // ledger never reports drift over the file that must not exist.
        if binary_name == "shall" {
            return Ok(());
        }

        let current_exe = tokio::task::spawn_blocking(std::env::current_exe)
            .await
            .map_err(|e| Error::Other(e.to_string()))?
            .map_err(|e| Error::Io(format!("Failed to locate shall binary: {}", e)))?;

        // Remove first: hard_link/copy onto an existing path fails, and a dangling symlink
        // reports as non-existent to `try_exists`, hence the explicit `is_symlink` check.
        if tokio::fs::try_exists(&target_path).await.unwrap_or(false) || target_path.is_symlink() {
            // S4: only overwrite a file Shall itself deployed. `bin_dir` is `~/.local/bin`,
            // shared with the user and every other tool; a same-named binary they put there is
            // an unmanaged file, and deploying a shim must not silently destroy it — the same
            // ownership rule `remove_shim` already follows. Redeploying Shall's own shim is
            // fine (it hashes identical to the shall binary).
            if !Self::is_deployed_shim(&target_path).await {
                return Err(Error::Refused(format!(
                    "refusing to deploy the `{}` shim: {:?} already exists and Shall did not \
                     create it. Move or rename that file yourself if you want the shim there.",
                    binary_name, target_path
                )));
            }
            fs::remove_file(&target_path).await.map_err(Error::from)?;
        }

        info!("Deploying shim for '{}' -> {:?}", binary_name, target_path);

        #[cfg(unix)]
        {
            // Hard link, never a symlink: `current_exe()` resolves symlinks, so a symlinked
            // shim would report the name "shall" and dispatch to itself instead of the
            // shimmed tool. Copy is the fallback since a link cannot cross filesystems.
            if let Err(e) = fs::hard_link(&current_exe, &target_path).await {
                debug!("Hard link failed ({}), falling back to copy...", e);
                fs::copy(&current_exe, &target_path)
                    .await
                    .map_err(Error::from)?;
            }
        }

        #[cfg(windows)]
        {
            fs::copy(&current_exe, &target_path)
                .await
                .map_err(Error::from)?;
        }

        if let Some(account) = &self.account {
            if let Err(e) = account.apply_owner(&target_path, false) {
                let _ = fs::remove_file(&target_path).await;
                return Err(e);
            }
        }

        Ok(())
    }

    /// `reaped` is proof the removal guard was consulted — see
    /// [`Reaped`](crate::app::sync::guard::Reaped). Unread here on purpose: its job is to be
    /// impossible to obtain without asking.
    pub async fn remove_shim(
        &self,
        binary_name: &str,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        // Was its own copy of the `.exe` rule, and the copy had drifted: it appended the
        // extension only when there was none, while `create_shim` replaced any extension that
        // was not `exe`. So `shim:tool.bat` deployed `tool.exe` and removal went looking for
        // `tool.bat`, leaving the shim behind and reporting success.
        let target_path = self.shim_path(binary_name);

        let present =
            tokio::fs::try_exists(&target_path).await.unwrap_or(false) || target_path.is_symlink();
        if !present {
            return Ok(());
        }
        if !Self::is_deployed_shim(&target_path).await {
            debug!("{:?} is not a Shall shim — leaving it alone.", target_path);
            return Ok(());
        }
        debug!("Removing shim {:?}", target_path);
        fs::remove_file(&target_path).await.map_err(Error::from)?;
        info!("Successfully removed shim '{}'", binary_name);
        Ok(())
    }

    /// Returns a list of all shims currently managed in the local bin directory.
    pub async fn list_shims(&self) -> Result<Vec<String>> {
        let mut shims = Vec::new();
        if !tokio::fs::try_exists(&self.bin_dir).await.unwrap_or(false) {
            return Ok(shims);
        }

        let mut entries = fs::read_dir(&self.bin_dir).await.map_err(Error::from)?;

        while let Some(entry) = entries.next_entry().await.map_err(Error::from)? {
            let path = entry.path();
            let metadata = entry.metadata().await.map_err(Error::from)?;

            if metadata.is_file() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name != "shall" && name != "shall.exe" && Self::is_deployed_shim(&path).await
                    {
                        #[cfg(windows)]
                        {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                shims.push(stem.to_string());
                            }
                        }
                        #[cfg(unix)]
                        {
                            shims.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(shims)
    }
}

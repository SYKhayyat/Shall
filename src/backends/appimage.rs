use crate::backends::artifact::teardown::{still_installed, tear_down, Deployed};
use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppImageState {
    pub url: String,
    pub local_path: String,
    pub symlink_path: String,
}

pub struct AppImageBackendCore {
    pub executor: CommandExecutor,
    /// `[guard] confine_bin`: whether a link name may reach outside `~/.local/bin` (SEC1).
    pub confine_bin: bool,
    /// K4: also clean the fetched AppImage from the cache locations on removal.
    pub clean_cache_on_remove: bool,
    pub cache_dirs: Vec<PathBuf>,
    pub install_dir: PathBuf,
    /// Where the `.AppImage` is linked from — `[bin_dir]`, the same directory the shims use and
    /// the one a sandboxed config moves (2026-07-29; it was built from `dirs::home_dir()` here).
    pub bin_dir: PathBuf,
    pub state_file: PathBuf,
    /// The deployment records, in memory, behind a lock.
    ///
    /// **The third of the three backends that keep a private state file, and the one that had
    /// no lock at all.** `github:` and `web:` at least took one for each half; here `install`
    /// read the whole file at the top, downloaded, and wrote the whole file at the bottom with
    /// nothing in between. Two `appimage:` lines in one wave both read `{}`, and whichever
    /// finished last wrote a map with only its own record. The record that lost describes a
    /// deployed binary and its symlink, so the package reads as unmanaged and teardown has
    /// nothing to remove.
    state: Mutex<Option<HashMap<String, AppImageState>>>,
}

impl AppImageBackendCore {
    pub fn new(
        executor: CommandExecutor,
        install_dir: PathBuf,
        bin_dir: PathBuf,
        confine_bin: bool,
        clean_cache_on_remove: bool,
        cache_dirs: Vec<PathBuf>,
    ) -> Self {
        let state = install_dir.join("state.json");

        Self {
            executor,
            confine_bin,
            clean_cache_on_remove,
            cache_dirs,
            install_dir,
            bin_dir,
            state_file: state,
            state: Mutex::new(None),
        }
    }

    async fn ensure_dirs(&self) -> Result<PathBuf> {
        if !tokio::fs::try_exists(&self.install_dir)
            .await
            .unwrap_or(false)
        {
            crate::utils::file::ensure_dir_async(&self.install_dir).await?;
        }
        if !tokio::fs::try_exists(&self.bin_dir).await.unwrap_or(false) {
            crate::utils::file::ensure_dir_async(&self.bin_dir).await?;
        }
        Ok(self.bin_dir.clone())
    }

    /// The records as they stand, for reading. A copy: holding a borrow would hold the lock
    /// across the download, which is the whole reason this backend is concurrent.
    pub async fn load_state(&self) -> Result<HashMap<String, AppImageState>> {
        let mut guard = self.state.lock().await;
        Ok(Self::loaded(&mut guard, &self.state_file).await?.clone())
    }

    /// Read the file into the memo the first time, and hand back the map either way.
    ///
    /// Through `ledger::load_json_records`, which is where the absent-versus-unparseable rule
    /// lives. Reading a corrupt file as an empty map is not a read failure that recovers — the
    /// emptiness is merged and written back, and the record of every deployed artifact is gone.
    async fn loaded<'a>(
        guard: &'a mut Option<HashMap<String, AppImageState>>,
        state_file: &Path,
    ) -> Result<&'a mut HashMap<String, AppImageState>> {
        if guard.is_none() {
            *guard = Some(crate::core::ledger::load_json_records(state_file).await?);
        }
        Ok(guard.as_mut().expect("just filled"))
    }

    /// Apply one task's changes to the shared records and write them, under one lock.
    ///
    /// A caller hands over what it changed, not the map it believes the file should hold, so a
    /// concurrent installer's records are merged rather than overwritten. Compact, not pretty,
    /// for the reason `core::state` gives about the registry beside it. The write is
    /// `write_atomic`, which is already `spawn_blocking` underneath and already answers to
    /// `--dry-run`.
    pub async fn commit_state(
        &self,
        installed: Vec<(String, AppImageState)>,
        removed: Vec<String>,
    ) -> Result<()> {
        let mut guard = self.state.lock().await;
        let map = Self::loaded(&mut guard, &self.state_file).await?;
        // **A preview changes the memo as little as it changes the disk.** The merge happens on
        // a copy, and the copy is only adopted for a run that acts. `persist` already refuses
        // the write under `--dry-run` and says "would write"; before this map existed, each
        // call re-read the file, so a preview's changes could not survive to the next question.
        // With a memo they would — and the next `list_installed` in the same run would report a
        // package the preview only said it would install.
        let mut merged = map.clone();
        for name in &removed {
            merged.remove(name);
        }
        merged.extend(installed);
        if !crate::core::dry_run::active() {
            *map = merged.clone();
        }
        let data = serde_json::to_string(&merged).map_err(Error::from)?;
        self.executor.write_atomic(&self.state_file, &data).await
    }
}

#[async_trait]
impl BackendCore for AppImageBackendCore {
    fn name(&self) -> &str {
        "appimage"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }

    fn needs_root(&self) -> bool {
        false
    }
}

#[async_trait]
impl MetadataProvider for AppImageBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct AppImageInstallable {
    pub core: Arc<AppImageBackendCore>,
}

#[async_trait]
impl Installable for AppImageInstallable {
    async fn install(&self, specs: &[PackageSpec], _: bool) -> Result<()> {
        let bin_dir = self.core.ensure_dirs().await?;
        let mut state = self.core.load_state().await?;
        // What this call changed, kept apart from the copy it reads.
        let mut installed_records: Vec<(String, AppImageState)> = Vec::new();

        for spec in specs {
            let url = &spec.name;
            // SEC2. This backend had no checksum option at all: it fetched any URL, chmod
            // 0755'd the result and put it on PATH, so `appimage:http://…` was one line
            // between a stranger and your shell.
            let allow_http = crate::core::download::allows_http(spec);
            crate::core::download::check_scheme(url, allow_http, url)?;
            crate::core::download::check_checksum_declared(spec)?;
            let client = crate::core::download::client(allow_http, "shall-manager")?;
            let filename = crate::utils::file::url_filename(url)?;
            let dest_path = self.core.install_dir.join(&filename);

            // Q37: the PATH name comes from the URL, so the refusal below can be asked now —
            // before the network — instead of after a download that was always going to be
            // thrown away. Same question, same message, no bytes.
            let link_name = filename
                .strip_suffix(".AppImage")
                .or_else(|| filename.strip_suffix(".appimage"))
                .unwrap_or(&filename);
            let download_only = crate::backends::artifact::ArtifactOptions::read(&spec.options)
                .map(|o| o.download_only)
                .unwrap_or(false);
            let link_path = if download_only {
                None
            } else {
                let link_path =
                    crate::utils::bin_destination(&bin_dir, link_name, self.core.confine_bin)?;
                crate::utils::ensure_deployable(
                    &link_path,
                    &self.core.install_dir,
                    state.get(&spec.name).map(|s| s.symlink_path.as_str()),
                )
                .await?;
                Some(link_path)
            };

            // Every refusal above is answered first, because that is what a preview is for:
            // an `@bin=` that escapes and a destination Shall does not own are told about here,
            // not after the transfer. Past this point there is only the transfer itself.
            if crate::core::dry_run::active() {
                crate::would!("download {} and install it as an AppImage", url);
                continue;
            }

            info!("AppImage: Downloading {}...", url);
            let response = client.get(url).send().await?;
            if !response.status().is_success() {
                return Err(Error::Other(format!(
                    "Download failed for {}: {}",
                    url,
                    response.status()
                )));
            }

            // **Nothing arrives at the artifact path until it is whole and verified**, and that is
            // now the download function's contract rather than this backend's habit. Streaming
            // onto `dest_path` meant a re-install whose connection dropped at 60% replaced a
            // working binary with a truncated one — and the PATH symlink from the previous
            // install already pointed there, so it followed. This line was the one place that
            // staged by hand (a `.shall-part` sibling, then a rename), and the hand-rolled copy
            // is gone: `stage_capped` streams beside the destination, `verify_checksum` reads
            // those bytes, and `commit` renames them into place.
            let staged = crate::core::download::stage_capped(response, &dest_path, url).await?;
            if staged.is_staged() {
                // Before the chmod below, never after: an unverified file must never exist as an
                // executable, even briefly.
                if let Some(expected) = spec.options.one("sha256") {
                    crate::core::verify_checksum(staged.path(), expected).await?;
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = tokio::fs::metadata(staged.path()).await?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    tokio::fs::set_permissions(staged.path(), perms).await?;
                }

                staged.commit().await?;
            }

            // D3b: `@download_only` keeps the fetched AppImage on disk but never links it onto
            // PATH. It is still declared, so a later removal still deletes the file.
            let symlink_path = match &link_path {
                None => String::new(),
                Some(link_path) => {
                    crate::utils::deploy_executable(
                        &dest_path,
                        link_path,
                        &self.core.install_dir,
                        state.get(&spec.name).map(|s| s.symlink_path.as_str()),
                    )
                    .await?;
                    info!(
                        "AppImage: Successfully installed {} to {}",
                        link_name,
                        link_path.display()
                    );
                    link_path.to_string_lossy().to_string()
                }
            };
            if download_only {
                info!(
                    "AppImage: fetched {} (download-only, not on PATH)",
                    filename
                );
            }

            let record = AppImageState {
                url: url.clone(),
                local_path: dest_path.to_string_lossy().to_string(),
                symlink_path,
            };
            installed_records.push((spec.name.clone(), record.clone()));
            state.insert(spec.name.clone(), record);
        }

        self.core
            .commit_state(installed_records, Vec::new())
            .await?;
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        _: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        let mut state = self.core.load_state().await?;

        let mut failures = Vec::new();
        // What this call changed, committed rather than the whole map.
        let mut removed_names: Vec<String> = Vec::new();
        for name in names {
            if let Some(info) = state.remove(name) {
                debug!("AppImage: Removing local files for {}", name);
                // A download-only AppImage was never linked, so `symlink_path` is empty and
                // there is no PATH entry to drop — which `Deployed::path` already knows.
                let deployed = Deployed::default()
                    .path(&info.local_path)
                    .path(&info.symlink_path)
                    .cached_url(&info.url);
                let errors = tear_down(
                    &deployed,
                    &self.core.executor,
                    self.core.clean_cache_on_remove,
                    &self.core.cache_dirs,
                )
                .await;
                if errors.is_empty() {
                    removed_names.push(name.clone());
                    info!("AppImage: Removed {}", name);
                } else {
                    // Still on disk and still on PATH: the name never joins `removed_names`,
                    // so the shared record it was taken from stands.
                    let _ = info;
                    failures.push(format!("{}: {}", name, errors.join("; ")));
                }
            } else {
                warn!("AppImage: No record found for {}, skipping removal.", name);
            }
        }

        self.core.commit_state(Vec::new(), removed_names).await?;
        if !failures.is_empty() {
            return Err(still_installed("AppImage", &failures));
        }
        Ok(())
    }
}

pub struct AppImageQueryable {
    pub core: Arc<AppImageBackendCore>,
}

#[async_trait]
impl Queryable for AppImageQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), "appimage")
    }

    /// **The name is the URL, because the URL is what `install` was handed and what the state
    /// is keyed by.** Reporting the basename instead meant `info(url)` never matched its own
    /// state file: every declared AppImage read as absent, so `sync` re-downloaded all of them
    /// on every run, for ever, and a removal could never find the row it was meant to delete.
    /// Same shape as `btrfs:`, fixed 2026-07-30, and `web:`, which never had it.
    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        let state = self.core.load_state().await?;
        Ok(state
            .keys()
            .map(|url| Package::new(url, "appimage"))
            .collect())
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        self.list_installed().await
    }

    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let all = self.installed_listing().await?;
        Ok(all.iter().find(|p| p.name == name).cloned())
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    cfg: &crate::config::Config,
) {
    let core = Arc::new(AppImageBackendCore::new(
        exec.clone(),
        cfg.appimage_dir.clone(),
        cfg.bin_dir.clone(),
        cfg.guard.confine_bin,
        cfg.clean_cache_on_remove,
        cfg.cache_dirs.clone(),
    ));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(AppImageInstallable { core: core.clone() }))
            .with_queryable(Arc::new(AppImageQueryable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}

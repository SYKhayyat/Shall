//! `dir:PATH` — idempotent directory creation with ownership and permissions (U71).
//!
//! Creates a directory (and parents) if absent, sets ownership and permissions. Teardown
//! removes the directory only if empty. Phase: Dependents (after packages).

use crate::core::{BackendCore, CommandExecutor, Error, Installable, PackageSpec, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

pub struct DirBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    pub config: Arc<crate::config::Config>,
}

pub fn parse_mode(value: &str) -> Result<u32> {
    if value.is_empty()
        || value.len() > 4
        || !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte))
    {
        return Err(Error::Validation(format!(
            "invalid mode `{value}` — use octal like 0700"
        )));
    }
    let mode = u32::from_str_radix(value, 8)
        .map_err(|_| Error::Validation(format!("invalid mode `{value}` — use octal like 0700")))?;
    if mode > 0o7777 {
        return Err(Error::Validation(format!(
            "invalid mode `{value}` — use octal like 0700"
        )));
    }
    Ok(mode)
}

impl DirBackendCore {
    pub fn new(executor: CommandExecutor, config: Arc<crate::config::Config>) -> Self {
        Self {
            executor,
            name: "dir".to_string(),
            config,
        }
    }
}

#[async_trait]
impl BackendCore for DirBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}

pub struct DirInstallable {
    pub core: Arc<DirBackendCore>,
    pub created: std::sync::Mutex<std::collections::BTreeSet<PathBuf>>,
}

impl DirInstallable {
    fn note_created(&self, path: &Path) {
        if let Ok(mut created) = self.created.lock() {
            created.insert(path.to_path_buf());
        }
    }
}

#[async_trait]
impl Installable for DirInstallable {
    async fn install(&self, specs: &[PackageSpec], _: bool) -> Result<()> {
        for spec in specs {
            let path_str = &spec.name;
            let user = spec.options.one("user");
            let owner = spec.options.one("owner");
            let mode = spec.options.one("mode");
            let mode_value = match mode {
                Some(value) => Some(parse_mode(value)?),
                None => None,
            };
            #[cfg(not(unix))]
            if mode_value.is_some() {
                return Err(Error::Unsupported(
                    "dir: mode ownership permissions are not supported on this platform".into(),
                ));
            }
            let account = match user {
                Some(name) => Some(crate::core::account::Account::resolve(name)?),
                None => None,
            };
            let desired_owner = owner.or(user);
            if let Some(name) = desired_owner {
                crate::core::account::Account::resolve(name)?;
            }
            let path = match &account {
                Some(account) => account.expand_home(path_str)?,
                None => super::link::resolve_target(path_str)?,
            };
            crate::core::extras_lock::require_absolute_dir(path_str, &path)?;

            if let Ok(metadata) = tokio::fs::symlink_metadata(&path).await {
                if !metadata.is_dir() {
                    return Err(Error::Validation(format!(
                        "`dir:{}` names {:?}, which is not a directory",
                        path_str, path
                    )));
                }
            }

            if self.core.executor.dry_run {
                if !path.exists() {
                    crate::would!("Dir: would create {:?}", path);
                    if let Some(name) = desired_owner {
                        super::link::chown_to_user(&path, name, &self.core.executor, true).await?;
                    }
                    if let Some(value) = mode_value {
                        crate::would!("Dir: would set mode {:o} on {:?}", value, path);
                    }
                } else {
                    if let Some(name) = desired_owner {
                        super::link::chown_to_user(&path, name, &self.core.executor, true).await?;
                    }
                    // A POSIX mode is a Unix question, and the answer on Windows is that there
                    // is none to answer. The `#[cfg]` was on the *body*, which left the binding
                    // alive on a target where nothing reads it — a warning, and CI rates warnings
                    // fatal. Putting it on the `if let` says the same thing and compiles.
                    #[cfg(unix)]
                    if let Some(value) = mode_value {
                        use std::os::unix::fs::PermissionsExt;
                        let current =
                            std::fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
                        if current != value {
                            crate::would!("Dir: would set mode {:o} on {:?}", value, path);
                        }
                    }
                }
                continue;
            }

            let created = !path.exists();
            if created {
                if self.core.config.link.auto_create_parent_dirs {
                    super::link::ensure_parent_dir_for_user(&path, &self.core.executor, user)
                        .await?;
                }
                tokio::fs::create_dir(&path).await.map_err(Error::from)?;
                info!("Dir: Created {:?}", path);
                self.note_created(&path);
            }

            if let Some(name) = desired_owner {
                super::link::chown_to_user(&path, name, &self.core.executor, true).await?;
            }

            #[cfg(unix)]
            if let Some(value) = mode_value {
                use std::os::unix::fs::PermissionsExt;
                let current = tokio::fs::metadata(&path)
                    .await
                    .map_err(Error::from)?
                    .permissions()
                    .mode()
                    & 0o7777;
                if current != value {
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(value))
                        .map_err(Error::from)?;
                }
            }
        }
        Ok(())
    }

    fn created_paths(&self) -> Vec<PathBuf> {
        match self.created.lock() {
            Ok(mut created) => std::mem::take(&mut *created).into_iter().collect(),
            Err(poisoned) => std::mem::take(&mut *poisoned.into_inner())
                .into_iter()
                .collect(),
        }
    }

    async fn remove(
        &self,
        names: &[String],
        _: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        let mut kept: Vec<String> = Vec::new();
        for name in names {
            let path = PathBuf::from(name);
            let metadata = match tokio::fs::symlink_metadata(&path).await {
                Ok(metadata) => metadata,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    kept.push(format!("{} could not be read: {e}", path.display()));
                    continue;
                }
            };
            if !metadata.is_dir() {
                return Err(Error::Validation(format!(
                    "`dir:{}` is not a directory",
                    path.display()
                )));
            }
            let mut retained = None;
            match tokio::fs::read_dir(&path).await {
                Ok(mut rd) => match rd.next_entry().await {
                    Ok(Some(_)) => {
                        tracing::warn!("Dir: {:?} is not empty — skipping removal", path);
                        retained = Some(format!("{} is not empty", path.display()));
                    }
                    Ok(None) => {}
                    Err(e) => retained = Some(format!("{} could not be read: {e}", path.display())),
                },
                Err(e) => retained = Some(format!("{} could not be read: {e}", path.display())),
            }
            if let Some(reason) = retained {
                if !self.core.executor.dry_run {
                    kept.push(reason);
                }
                continue;
            }
            if self.core.executor.dry_run {
                crate::would!("Dir: would remove empty {:?}", path);
                continue;
            }
            tokio::fs::remove_dir(&path).await.map_err(Error::from)?;
            info!("Dir: Removed empty {:?}", path);
        }
        if kept.is_empty() {
            return Ok(());
        }
        Err(Error::Validation(format!(
            "{} left in place: {}",
            kept.len(),
            kept.join("; ")
        )))
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    cfg: &crate::config::Config,
) {
    let core = Arc::new(DirBackendCore::new(exec.clone(), Arc::new(cfg.clone())));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(DirInstallable {
                core,
                created: Default::default(),
            }))
            .build(),
    ));
}

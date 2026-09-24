//! `dir:PATH` — idempotent directory creation with ownership and permissions (U71).
//!
//! Creates a directory (and parents) if absent, sets ownership and permissions. Teardown
//! removes the directory only if empty. Phase: Dependents (after packages).

use crate::core::{BackendCore, CommandExecutor, Error, Installable, PackageSpec, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

pub struct DirBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    pub config: Arc<crate::config::Config>,
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
}

#[async_trait]
impl Installable for DirInstallable {
    async fn install(&self, specs: &[PackageSpec], _: bool) -> Result<()> {
        for spec in specs {
            let path_str = &spec.name;
            let user = spec.options.one("user");
            let owner = spec.options.one("owner");
            let mode = spec.options.one("mode");

            // U71: when @user=NAME is present, resolve ~/ to that user's home.
            let path = match user {
                Some(u) => super::link::resolve_target_for_user(path_str, u)?,
                None => super::link::resolve_target(path_str)?,
            };

            if self.core.executor.dry_run {
                if path.exists() {
                    crate::would!("Dir: {:?} already exists", path);
                } else {
                    crate::would!("Dir: would create {:?}", path);
                }
                if let Some(m) = mode {
                    crate::would!("Dir: would set mode {} on {:?}", m, path);
                }
                if let Some(o) = owner {
                    crate::would!("Dir: would chown {:?} to {}", path, o);
                }
                continue;
            }

            // Create the directory (and parents) if it doesn't exist, gated by config.
            if !path.exists() {
                if self.core.config.link.auto_create_parent_dirs {
                    tokio::fs::create_dir_all(&path)
                        .await
                        .map_err(Error::from)?;
                } else {
                    tokio::fs::create_dir(&path)
                        .await
                        .map_err(Error::from)?;
                }
                info!("Dir: Created {:?}", path);
            }

            // Set mode if specified.
            if let Some(m) = mode {
                let mode_val = u32::from_str_radix(m, 8).map_err(|_| {
                    Error::Other(format!(
                        "invalid mode `{}` for dir:{:?} — use octal like 0700",
                        m, path
                    ))
                })?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode_val))
                        .map_err(Error::from)?;
                }
            }

            // Set ownership if specified.
            if let Some(o) = owner {
                super::link::chown_to_user(&path, o, &self.core.executor).await?;
            }
        }
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        _: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        for name in names {
            let path = std::path::PathBuf::from(name);
            if !path.exists() {
                continue;
            }
            match std::fs::read_dir(&path) {
                Ok(mut rd) => {
                    if rd.next().is_some() {
                        tracing::warn!("Dir: {:?} is not empty — skipping removal", path);
                        continue;
                    }
                }
                Err(_) => continue,
            }
            if self.core.executor.dry_run {
                crate::would!("Dir: would remove empty {:?}", path);
                continue;
            }
            tokio::fs::remove_dir(&path)
                .await
                .map_err(Error::from)?;
            info!("Dir: Removed empty {:?}", path);
        }
        Ok(())
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
            .with_installable(Arc::new(DirInstallable { core }))
            .build(),
    ));
}

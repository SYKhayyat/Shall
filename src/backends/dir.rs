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
    created: std::sync::Mutex<std::collections::BTreeSet<PathBuf>>,
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
                    if let Some(value) = mode_value {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let current =
                                std::fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
                            if current != value {
                                crate::would!("Dir: would set mode {:o} on {:?}", value, path);
                            }
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

            if let Some(value) = mode_value {
                #[cfg(unix)]
                {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::Account;
    use tempfile::tempdir;

    fn spec(name: &str, options: &[(&str, &str)]) -> PackageSpec {
        let mut opts = crate::config::grammar::Options::default();
        for (key, value) in options {
            opts.set(*key, *value);
        }
        PackageSpec {
            name: name.to_string(),
            backend: "dir".into(),
            options: opts,
            requires: Vec::new(),
            present: true,
        }
    }

    fn installer() -> DirInstallable {
        let executor = CommandExecutor::new(false, false);
        let core = Arc::new(DirBackendCore::new(
            executor,
            Arc::new(crate::config::Config::default()),
        ));
        DirInstallable {
            core,
            created: Default::default(),
        }
    }

    #[tokio::test]
    async fn a_directory_that_cannot_be_emptied_is_a_failure_and_not_a_removal() {
        let root = tempdir().unwrap();
        let busy = root.path().join("busy");
        std::fs::create_dir(&busy).unwrap();
        std::fs::write(busy.join("the-users-data"), "keep me\n").unwrap();

        let error = installer()
            .remove(
                &[busy.display().to_string()],
                false,
                crate::app::sync::guard::Reaped::for_reason(
                    crate::app::sync::guard::GuardScope::Sync,
                    "a unit test of the teardown itself",
                ),
            )
            .await
            .expect_err("a directory that was left behind is not a successful removal");
        assert!(error.to_string().contains("not empty"), "{error}");
        assert!(
            busy.join("the-users-data").exists(),
            "the contents were destroyed by a teardown that could not remove the directory"
        );
    }

    #[tokio::test]
    async fn an_absent_directory_is_a_successful_removal() {
        let root = tempdir().unwrap();
        installer()
            .remove(
                &[root.path().join("never-existed").display().to_string()],
                false,
                crate::app::sync::guard::Reaped::for_reason(
                    crate::app::sync::guard::GuardScope::Sync,
                    "a unit test of the teardown itself",
                ),
            )
            .await
            .expect("there is nothing left to remove, which is the state the caller wanted");
    }

    #[tokio::test]
    async fn a_dir_refuses_a_file_at_its_path() {
        let root = tempdir().unwrap();
        let path = root.path().join("not-a-directory");
        std::fs::write(&path, "file").unwrap();
        let error = installer()
            .install(&[spec(path.to_str().unwrap(), &[])], false)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("not a directory"));
        assert_eq!(std::fs::read(&path).unwrap(), b"file");
    }

    #[tokio::test]
    async fn invalid_mode_is_refused_before_creating_a_directory() {
        let root = tempdir().unwrap();
        let path = root.path().join("invalid");
        let error = installer()
            .install(&[spec(path.to_str().unwrap(), &[("mode", "10000")])], false)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("invalid mode"));
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn the_creation_receipt_is_taken_once_and_does_not_outlive_its_run() {
        let root = tempdir().unwrap();
        let made = root.path().join("made");
        let installer = installer();
        installer
            .install(&[spec(made.to_str().unwrap(), &[])], false)
            .await
            .unwrap();
        assert_eq!(installer.created_paths(), vec![made.clone()]);
        assert!(
            installer.created_paths().is_empty(),
            "a second reader saw a receipt the first had already been given, so a reconcile \
             that ran twice would read the same directory as made twice over"
        );

        installer
            .install(&[spec(made.to_str().unwrap(), &[])], false)
            .await
            .unwrap();
        assert!(installer.created_paths().is_empty());
    }

    #[tokio::test]
    async fn a_relative_dir_is_refused_rather_than_read_against_the_working_directory() {
        for declared in ["relative/state", ".", "./state", "../state"] {
            let before = std::env::current_dir().unwrap();
            let error = installer()
                .install(&[spec(declared, &[])], false)
                .await
                .expect_err("a relative dir: must not be resolved against the process");
            let message = error.to_string();
            assert!(message.contains(&format!("dir:{declared}")), "{message}");
            assert!(message.contains("not an absolute path"), "{message}");
            assert_eq!(std::env::current_dir().unwrap(), before);
        }
    }

    #[test]
    fn a_relative_dir_is_refused_at_key_construction_too() {
        for declared in ["relative/state", "."] {
            let error = crate::core::extras_lock::extra_key(
                &crate::config::grammar::Statement::Dir(declared.to_string(), Default::default()),
            )
            .expect_err("a relative dir: has no stable key")
            .to_string();
            assert!(error.contains(&format!("dir:{declared}")), "{error}");
            assert!(error.contains("not an absolute path"), "{error}");
        }
    }

    #[tokio::test]
    async fn a_home_relative_dir_is_still_resolved() {
        let account = Account::current().unwrap();
        let installer = installer();
        let declaration = spec(
            &format!("~/.local/state/shall-abs-test-{}", std::process::id()),
            &[],
        );
        installer.install(&[declaration], false).await.unwrap();
        let resolved = account
            .home()
            .join(".local/state")
            .join(format!("shall-abs-test-{}", std::process::id()));
        assert!(resolved.is_dir(), "{}", resolved.display());
        let _ = std::fs::remove_dir(&resolved);
    }

    #[tokio::test]
    async fn a_per_user_dir_is_created_and_converges_without_reapplying_ownership() {
        let root = tempdir().unwrap();
        let account = Account::current().unwrap();
        let path = root.path().join("nested").join("state");
        let declaration = spec(
            path.to_str().unwrap(),
            &[("user", account.name()), ("mode", "0750")],
        );
        std::fs::create_dir_all(&path).unwrap();
        let installer = installer();
        installer
            .install(std::slice::from_ref(&declaration), false)
            .await
            .unwrap();
        assert!(path.is_dir());
        assert!(account.owns(&path, true).unwrap());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
                0o750
            );
        }
        installer.install(&[declaration], false).await.unwrap();
        assert!(path.is_dir());
    }
}

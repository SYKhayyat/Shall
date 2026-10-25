//! Declared storage objects: ZFS datasets and LVM logical volumes, one family (U30).
//!
//! `btrfs:` already declares a btrfs subvolume as an object; a ZFS dataset (`zfs create`) and an
//! LVM logical volume (`lvcreate`) are the same idea — a declared, sized, mounted storage object
//! — so they are one family rather than three unrelated backends. They are Rust rather than a
//! `ManagerConfig` because they are not argv-with-`{name}={version}`: a volume has a size and a
//! mountpoint, not a version.
//!
//! **The safety edge U30 turns on: a `remove` here destroys a filesystem and everything on it.**
//! `zfs destroy` and `lvremove` are not "uninstall a package"; they are "erase a disk". Both are
//! ordinary backends, so their removals run through the **normal** sync guard with no special
//! escalation — which is exactly the point: a declared volume is protectable like a package
//! (`[guard] protected_packages` matches `zfs:tank/data`), it counts against `max_removals`, and
//! deleting the line previews the destruction before the guard lets it proceed. A storage backend
//! that ran its own removal outside the guard would be the teleport bug (V-lesson 2026-07-17) with
//! a filesystem on the end of it.

use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// What a storage backend reports for an object it read and found no limit on — as against
/// reporting nothing, which means it could not read (Q19).
pub const NO_LIMIT: &str = "none";

// --- ZFS datasets -----------------------------------------------------------------------------

/// The argv to create a ZFS dataset. Pure, so the command is unit-testable without a pool.
pub fn zfs_create(name: &str) -> Vec<String> {
    vec!["create".into(), name.into()]
}

/// `zfs set quota=<size> <name>` — a property, set after create.
pub fn zfs_set(property: &str, value: &str, name: &str) -> Vec<String> {
    vec!["set".into(), format!("{}={}", property, value), name.into()]
}

/// `zfs list -H -p -o name,quota,mountpoint`, as datasets with their quota and where each is
/// mounted.
///
/// `-H` separates columns with a tab and nothing else, so the split is on `\t`: a mountpoint is
/// a path and may contain spaces.
///
/// `-p` is what makes the quota comparable: it prints the exact byte count instead of `10G`, so
/// a declared `@quota=10240M` and a reported `10G` are one number rather than two strings that
/// have to be talked into agreeing (Q19). ZFS spells "no quota" as `0` under `-p`, and a `0` is
/// recorded as no quota rather than as a zero-byte limit.
///
/// A dataset reporting `none`, `legacy` or `-` has **no mountpoint ZFS is managing**, and that is
/// recorded as no property at all rather than as those literal words — a declaration asking for
/// `@mount=/srv` against a dataset ZFS is not mounting is unsatisfied, and the planner has to be
/// able to see that. Reporting `legacy` as if it were a path would make the comparison false and
/// re-run `zfs set mountpoint=` on every sync.
pub fn parse_zfs_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let name = cols.next()?.trim();
            if name.is_empty() {
                return None;
            }
            let mut p = Package::new(name, "zfs");
            if let Some(q) = cols.next().map(str::trim) {
                // Three states, not two: a byte count, `none` for a dataset ZFS read and found no
                // limit on, and — only when the listing itself could not run — no property at all.
                // A missing property means "unknown" everywhere downstream, and an unknown that
                // read as "no limit" would re-apply a quota on every sync for ever.
                let limit = q.parse::<u64>().unwrap_or(0);
                p.properties.insert(
                    "quota".into(),
                    if limit > 0 {
                        limit.to_string()
                    } else {
                        NO_LIMIT.into()
                    },
                );
            }
            if let Some(m) = cols
                .next()
                .map(str::trim)
                .filter(|m| !matches!(*m, "" | "-" | "none" | "legacy"))
            {
                p.properties.insert("mount".to_string(), m.to_string());
            }
            Some(p)
        })
        .collect()
}

/// `zfs destroy -r <name>` — destroys the dataset and its children. The dangerous verb; it runs
/// only after the sync guard has cleared the removal.
pub fn zfs_destroy(name: &str) -> Vec<String> {
    vec!["destroy".into(), "-r".into(), name.into()]
}

pub struct ZfsBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
}

impl ZfsBackendCore {
    pub fn new(executor: CommandExecutor) -> Self {
        Self {
            executor,
            name: "zfs".to_string(),
        }
    }
    async fn run(&self, args: &[String], sudo: bool) -> Result<()> {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.executor.run("zfs", &refs, sudo).await.map(|_| ())
    }
}

#[async_trait]
impl BackendCore for ZfsBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        self.executor.command_exists_sync("zfs")
    }
    fn probes(&self) -> Vec<String> {
        vec!["zfs".into()]
    }
    fn needs_root(&self) -> bool {
        true
    }
}

#[async_trait]
impl MetadataProvider for ZfsBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct ZfsInstallable {
    pub core: Arc<ZfsBackendCore>,
}

#[async_trait]
impl Installable for ZfsInstallable {
    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        for spec in specs {
            let name = &spec.name;
            // Create only if absent, so a sync is idempotent — `zfs list` answers existence.
            let exists = self
                .core
                .executor
                .run_output("zfs", &["list", "-H", "-o", "name", name], false)
                .await
                .map(|o| !o.trim().is_empty())
                .unwrap_or(false);
            if !exists {
                info!("ZFS: creating dataset {}", name);
                self.core.run(&zfs_create(name), sudo).await?;
            }
            if let Some(quota) = spec.options.one("quota") {
                self.core.run(&zfs_set("quota", quota, name), sudo).await?;
            }
            if let Some(mount) = spec.options.one("mount") {
                self.core
                    .run(&zfs_set("mountpoint", mount, name), sudo)
                    .await?;
            }
        }
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        for name in names {
            // Already gone is done — the same convergence the filesystem side of this family
            // has always had. **Asked, not guessed**: the question is the one this backend's
            // own install asks, and an ask that fails propagates rather than reading as "not
            // there", or a listing outage would skip destroys in silence.
            let out = self
                .core
                .executor
                .run_output("zfs", &["list", "-H", "-o", "name", name], false)
                .await?;
            if out.trim().is_empty() {
                debug!("ZFS: {} is already absent.", name);
                continue;
            }
            info!("ZFS: destroying dataset {}", name);
            self.core.run(&zfs_destroy(name), sudo).await?;
        }
        Ok(())
    }
}

pub struct ZfsQueryable {
    pub core: Arc<ZfsBackendCore>,
}

#[async_trait]
impl Queryable for ZfsQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        // `quota` and `mountpoint` ride along on the listing that already runs, so `sync` can tell
        // a declared `@mount=`/`@quota=` that took effect from one that did not — two more
        // columns, not two more subprocesses.
        let out = self
            .core
            .executor
            .run_output(
                "zfs",
                &["list", "-H", "-p", "-o", "name,quota,mountpoint"],
                false,
            )
            .await?;
        Ok(parse_zfs_list(&out))
    }
    async fn list_manual(&self) -> Result<Vec<Package>> {
        self.list_installed().await
    }
    async fn info(&self, name: &str) -> Result<Option<Package>> {
        Ok(self
            .installed_listing()
            .await?
            .iter()
            .find(|p| p.name == name)
            .cloned())
    }
}

// --- LVM logical volumes ----------------------------------------------------------------------

/// Split `vg/lv` into its volume group and logical volume. An LVM object is named `group/volume`,
/// the one spelling both `lvcreate` and `lvremove` accept via their own conventions.
pub fn split_lvm(name: &str) -> Result<(&str, &str)> {
    name.split_once('/')
        .filter(|(vg, lv)| !vg.is_empty() && !lv.is_empty())
        .ok_or_else(|| {
            Error::Validation(format!(
                "`lvm:{}` is not a volume — name it `group/volume`, e.g. `lvm:vg0/data`.",
                name
            ))
        })
}

/// `lvcreate -n <lv> -L <size> <vg>` — a logical volume needs a size; without one there is
/// nothing to create, which is a refusal, not a guess.
pub fn lvm_create(vg: &str, lv: &str, size: &str) -> Vec<String> {
    vec!["-n".into(), lv.into(), "-L".into(), size.into(), vg.into()]
}

/// `lvremove -y <vg>/<lv>` — destroys the volume. The dangerous verb, run only past the guard.
pub fn lvm_remove(vg: &str, lv: &str) -> Vec<String> {
    vec!["-y".into(), format!("{}/{}", vg, lv)]
}

/// `lvextend --resizefs -L <size> <vg>/<lv>` — grow a volume to its declared size (Q19).
///
/// **`--resizefs` is not optional here.** A volume grown without its filesystem gives the user
/// nothing they can see: `lvs` reports 20G and `df` still reports 10G, so the declaration reads as
/// applied while the space it asked for is unreachable. A volume carrying no filesystem at all
/// fails this loudly instead — `fsadm` cannot name a type — which is the honest limit of growing
/// by declaration, and better than silently applying half of one.
fn lvm_extend(vg: &str, lv: &str, size: &str) -> Vec<String> {
    vec![
        "--resizefs".into(),
        "-L".into(),
        size.into(),
        format!("{}/{}", vg, lv),
    ]
}

/// `lvreduce --resizefs --yes -L <size> <vg>/<lv>` — shrink, behind `@allow_shrink` (Q19).
///
/// `--resizefs` is what keeps this from being data loss: it shrinks the filesystem *before* the
/// volume, so the bytes being given up are ones nothing is using. Without it `lvreduce` truncates
/// a live filesystem, which is the destruction the flag is there to make deliberate — and it is
/// the same reason xfs is refused here by the tool rather than by us: xfs cannot shrink, `fsadm`
/// fails first, and the volume is never touched.
fn lvm_reduce(vg: &str, lv: &str, size: &str) -> Vec<String> {
    vec![
        "--resizefs".into(),
        "--yes".into(),
        "-L".into(),
        size.into(),
        format!("{}/{}", vg, lv),
    ]
}

/// What a declared `@size` means for a volume that already exists (Q19): the command to run, or
/// `None` when the volume is already the size the line asks for.
///
/// **Growing and shrinking are two decisions, not one command with a sign.** Growing hands back
/// space nothing was using; shrinking takes space away from a live filesystem, and on one that
/// cannot shrink at all it takes away whatever was past the new end. So the owner's ruling —
/// a declaration resizes, and the direction that can lose data says so on the line — lands here
/// as a refusal that names both sizes rather than a silent no-op or a silent truncation.
///
/// Pure, so the decision is testable without a volume group: the argv was never the hard part.
pub fn resize_plan(
    vg: &str,
    lv: &str,
    declared: &str,
    current: u64,
    allow_shrink: bool,
) -> Result<Option<(&'static str, Vec<String>)>> {
    let Some(want) = crate::core::parse_size(declared) else {
        return Err(Error::Validation(format!(
            "`lvm:{}/{}@size={}` is not a size — write it as a number and a unit, e.g. `10G` or \
             `500M`.",
            vg, lv, declared
        )));
    };
    if want == current {
        return Ok(None);
    }
    if want > current {
        info!(
            "LVM: growing {}/{} from {} to {}",
            vg,
            lv,
            crate::core::format_size(current),
            crate::core::format_size(want)
        );
        return Ok(Some(("lvextend", lvm_extend(vg, lv, declared))));
    }
    if !allow_shrink {
        return Err(Error::Validation(format!(
            "`lvm:{}/{}` is {} and the line declares {} — Shall will not shrink a volume unless \
             the line says so. Shrinking takes space back off a live filesystem, and one that \
             cannot be shrunk at all (xfs) loses whatever is past the new end. Add \
             `@allow_shrink=true` to mean it, or restore `@size={}`.",
            vg,
            lv,
            crate::core::format_size(current),
            crate::core::format_size(want),
            crate::core::format_size(current)
        )));
    }
    warn!(
        "LVM: shrinking {}/{} from {} to {} — the line carries @allow_shrink",
        vg,
        lv,
        crate::core::format_size(current),
        crate::core::format_size(want)
    );
    Ok(Some(("lvreduce", lvm_reduce(vg, lv, declared))))
}

/// `lvs --noheadings --units b --nosuffix -o vg_name,lv_name,lv_size`, as volumes and their size
/// in bytes.
///
/// Bytes, not `10.00g`: the reported side is never parsed as a display string (`core::size`).
pub fn parse_lvs_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let (vg, lv) = (cols.next()?, cols.next()?);
            let mut p = Package::new(format!("{}/{}", vg, lv), "lvm");
            if let Some(bytes) = cols.next().and_then(|b| b.parse::<u64>().ok()) {
                p.properties.insert("size".to_string(), bytes.to_string());
            }
            Some(p)
        })
        .collect()
}

pub struct LvmBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
}

impl LvmBackendCore {
    pub fn new(executor: CommandExecutor) -> Self {
        Self {
            executor,
            name: "lvm".to_string(),
        }
    }
}

#[async_trait]
impl BackendCore for LvmBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        self.executor.command_exists_sync("lvs")
    }
    /// `lvs`, not `lvm`. The default message said "Binary for lvm not found in PATH" and named
    /// a program this backend never looks for.
    fn probes(&self) -> Vec<String> {
        vec!["lvs".into()]
    }
    fn needs_root(&self) -> bool {
        true
    }
}

#[async_trait]
impl MetadataProvider for LvmBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct LvmInstallable {
    pub core: Arc<LvmBackendCore>,
}

impl LvmInstallable {
    /// This volume's size in bytes, or `None` if there is no such volume.
    ///
    /// Absence and size come from one question, because they are one question: `lvs` on a volume
    /// that is not there exits non-zero, and asking existence and size separately leaves a window
    /// where the answers disagree.
    async fn current_size(&self, vg: &str, lv: &str) -> Option<u64> {
        let path = format!("{}/{}", vg, lv);
        let out = self
            .core
            .executor
            .run_output(
                "lvs",
                &[
                    "--noheadings",
                    "--units",
                    "b",
                    "--nosuffix",
                    "-o",
                    "lv_size",
                    &path,
                ],
                false,
            )
            .await
            .ok()?;
        out.split_whitespace().next()?.parse().ok()
    }
}

#[async_trait]
impl Installable for LvmInstallable {
    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        for spec in specs {
            let (vg, lv) = split_lvm(&spec.name)?;
            let current = self.current_size(vg, lv).await;
            let Some(size) = spec.options.one("size") else {
                if current.is_some() {
                    continue;
                }
                return Err(Error::Validation(format!(
                    "`lvm:{}` has no `size` — a logical volume needs one to be created, e.g. \
                     `lvm:{}@size=10G`.",
                    spec.name, spec.name
                )));
            };
            let Some(current) = current else {
                info!("LVM: creating logical volume {}/{} ({})", vg, lv, size);
                let args = lvm_create(vg, lv, size);
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.core.executor.run("lvcreate", &refs, sudo).await?;
                continue;
            };
            let allow_shrink = spec.options.one("allow_shrink") == Some("true");
            let Some((program, args)) = resize_plan(vg, lv, size, current, allow_shrink)? else {
                continue;
            };
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.core.executor.run(program, &refs, sudo).await?;
        }
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        for name in names {
            let (vg, lv) = split_lvm(name)?;
            // Same ask-first tolerance as the zfs twin above: absent is done, unverifiable
            // refuses, present removes.
            let path = format!("{vg}/{lv}");
            let out = self
                .core
                .executor
                .run_output(
                    "lvs",
                    &[
                        "--noheadings",
                        "--units",
                        "b",
                        "--nosuffix",
                        "-o",
                        "vg_name,lv_name",
                        &path,
                    ],
                    false,
                )
                .await?;
            if out.trim().is_empty() {
                debug!("LVM: {path} is already absent.");
                continue;
            }
            info!("LVM: removing logical volume {}/{}", vg, lv);
            let args = lvm_remove(vg, lv);
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.core.executor.run("lvremove", &refs, sudo).await?;
        }
        Ok(())
    }
}

pub struct LvmQueryable {
    pub core: Arc<LvmBackendCore>,
}

#[async_trait]
impl Queryable for LvmQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        // `lv_size` rides along on the listing that already runs, in bytes, so `sync` can see a
        // declared `@size=` that no longer matches the volume (Q19).
        let out = self
            .core
            .executor
            .run_output(
                "lvs",
                &[
                    "--noheadings",
                    "--units",
                    "b",
                    "--nosuffix",
                    "-o",
                    "vg_name,lv_name,lv_size",
                ],
                false,
            )
            .await?;
        Ok(parse_lvs_list(&out))
    }
    async fn list_manual(&self) -> Result<Vec<Package>> {
        self.list_installed().await
    }
    async fn info(&self, name: &str) -> Result<Option<Package>> {
        Ok(self
            .installed_listing()
            .await?
            .iter()
            .find(|p| p.name == name)
            .cloned())
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    _cfg: &crate::config::Config,
) {
    let zfs = Arc::new(ZfsBackendCore::new(exec.clone()));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(zfs.clone())
            .with_installable(Arc::new(ZfsInstallable { core: zfs.clone() }))
            .with_queryable(Arc::new(ZfsQueryable { core: zfs.clone() }))
            .with_metadata_provider(zfs.clone())
            .build(),
    ));

    let lvm = Arc::new(LvmBackendCore::new(exec.clone()));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(lvm.clone())
            .with_installable(Arc::new(LvmInstallable { core: lvm.clone() }))
            .with_queryable(Arc::new(LvmQueryable { core: lvm.clone() }))
            .with_metadata_provider(lvm.clone())
            .build(),
    ));
}

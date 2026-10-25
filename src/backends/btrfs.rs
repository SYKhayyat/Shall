use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct BtrfsBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    /// The kernel's mount table. A field so a test can hand it a fixture — the alternative is
    /// a check that can only run on a Linux box that already has btrfs, which is every box
    /// this backend was never tested on.
    pub mounts_file: std::path::PathBuf,
    /// Where a declared mount is recorded. A field for the same reason, and the stakes are
    /// higher: this is the file a wrong edit stops the machine booting from.
    pub fstab_file: std::path::PathBuf,
}

/// One btrfs filesystem, as the mount table describes it.
///
/// `prefix` is the `subvol=` mount option, and it is not decoration: `btrfs subvolume list`
/// reports a path relative to the *filesystem* root, while `install` was handed a path on the
/// *mounted* tree. On a root mounted at `subvol=/@`, the two never name the same thing without
/// it — and a name `list` reports differently from the one `install` was given is a package
/// `sync` re-creates on every run, for ever.
///
/// `device` is what tells one filesystem from another. Two btrfs filesystems can each hold a
/// subvolume called `data`, and one filesystem can be mounted twice — so a subvolume's identity
/// is the device it lives on plus its path from that filesystem's root, never the path alone.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BtrfsMount {
    pub device: String,
    pub point: String,
    pub prefix: String,
}

/// Every btrfs filesystem in a mount table.
/// **`pub` for the integration tests' sake.** The btrfs mirror parses fixture mount tables with
/// the production parser rather than duplicating its escape and deduplication rules.
pub fn btrfs_mounts_in(table: &str) -> Vec<BtrfsMount> {
    let mut out = Vec::new();
    for line in table.lines() {
        let mut f = line.split_whitespace();
        let (dev, point, fstype, opts) = (f.next(), f.next(), f.next(), f.next());
        if fstype != Some("btrfs") {
            continue;
        }
        let (Some(dev), Some(point)) = (dev, point) else {
            continue;
        };
        // /proc/mounts octal-escapes whitespace in the mount point.
        let point = point.replace("\\040", " ").replace("\\011", "\t");
        let prefix = opts
            .unwrap_or("")
            .split(',')
            .find_map(|o| o.strip_prefix("subvol="))
            .unwrap_or("/")
            .trim_start_matches('/')
            .to_string();
        out.push(BtrfsMount {
            device: dev.to_string(),
            point,
            prefix,
        });
    }
    out.sort();
    out.dedup();
    out
}

/// One `btrfs subvolume list` report, as `(path from the filesystem root, name `install` would
/// accept)`. The first is the identity, the second is what a user reads.
fn subvolume_paths(mount: &str, prefix: &str, output: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in output.lines() {
        let Some(rel) = line.split(" path ").nth(1) else {
            continue;
        };
        let rel = rel.trim();
        // Only what is reachable *through this mount*. On a filesystem mounted at
        // `subvol=/@`, a sibling subvolume `@home` exists and has no path from here, so
        // reporting it would name something no verb could act on.
        //
        // A PATH prefix, not a string prefix: `@home` starts with `@` and is not under it.
        // My first version used `strip_prefix` alone and reported the sibling as `/home`,
        // which is a real directory on the same machine and the wrong one.
        let under = if prefix.is_empty() {
            rel
        } else if rel == prefix {
            ""
        } else if let Some(rest) = rel.strip_prefix(prefix).filter(|r| r.starts_with('/')) {
            rest
        } else {
            continue;
        };
        let under = under.trim_start_matches('/');
        let name = if under.is_empty() {
            mount.to_string()
        } else {
            format!("{}/{}", mount.trim_end_matches('/'), under)
        };
        out.push((rel.trim_start_matches('/').to_string(), name));
    }
    out
}

/// The `subvol=` an fstab entry must carry for a subvolume declared at `path`, and the mount
/// point it hangs under.
///
/// The inverse of [`subvolume_paths`], and it exists for the same reason: fstab names a
/// subvolume relative to the *filesystem* root, while a declaration names it on the *mounted*
/// tree. An entry built from the declared path mounts nothing on a filesystem whose own root is
/// a subvolume — there is no `subvol=/mnt/data/srv`; `subvol=/@/srv` is that same object.
///
/// Returns the `subvol=` **and the mount point of the filesystem holding it**. The second is
/// not a convenience: `btrfs filesystem show` answers for a *filesystem* and refuses a
/// subvolume — `not a valid btrfs filesystem: /mnt/fs/canary`, measured — so the UUID an fstab
/// entry needs can only be asked for at the mount point, never at the declared path.
///
/// `None` when the path is not under any btrfs mount, and when it *is* one: the root of a
/// mounted filesystem is not a subvolume this backend created, and writing an fstab entry for
/// it would rewrite the line that mounts the machine.
pub fn subvol_arg(mounts: &[BtrfsMount], path: &str) -> Option<(String, String)> {
    let path = path.trim_end_matches('/');
    let mount = mounts
        .iter()
        .filter(|m| {
            path.strip_prefix(m.point.trim_end_matches('/'))
                .is_some_and(|rest| rest.starts_with('/'))
        })
        .max_by_key(|m| m.point.trim_end_matches('/').len())?;
    let rest = path
        .strip_prefix(mount.point.trim_end_matches('/'))?
        .trim_start_matches('/');
    let prefix = mount.prefix.trim_matches('/');
    let subvol = if prefix.is_empty() {
        format!("/{}", rest)
    } else {
        format!("/{}/{}", prefix, rest)
    };
    Some((subvol, mount.point.clone()))
}

/// The filesystem UUID in a `btrfs filesystem show` report.
///
/// It is a token *within* the first line, not the start of one: the real report reads
/// `Label: none  uuid: 3b5f…`. This looked for a line beginning `uuid:`, found none, and failed
/// every time — which nobody saw, because the only caller is the `@mount` path and no
/// declaration could carry `@mount` until Q18 was ruled.
pub fn fs_uuid_in(output: &str) -> Option<String> {
    let mut fields = output.split_whitespace();
    while let Some(field) = fields.next() {
        if field == "uuid:" {
            return fields.next().map(str::to_string);
        }
        if let Some(uuid) = field.strip_prefix("uuid:").filter(|u| !u.is_empty()) {
            return Some(uuid.to_string());
        }
    }
    None
}

/// The mount point field of one fstab line, or `None` for a comment or a blank.
fn fstab_mount_point(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    line.split_whitespace().nth(1)
}

/// The `subvol=` of one fstab line, when the line mounts btrfs.
fn fstab_subvol(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let mut fields = line.split_whitespace();
    if fields.nth(2) != Some("btrfs") {
        return None;
    }
    fields
        .next()?
        .split(',')
        .find_map(|o| o.strip_prefix("subvol="))
}

/// The option field of this subvolume's fstab entry, without the `subvol=` that addresses it.
///
/// The inverse of what [`fstab_with`] writes, so a declared `@mount_options` can be compared
/// against the entry on disk (Q19). Without it a changed `@mount_options` is invisible: the
/// subvolume exists and is mounted where the line says, so `sync` finds nothing to do and the
/// old options survive every run — the same shape as a `@quota` that never re-applies, one file
/// over.
pub fn fstab_options(content: &str, subvol: &str) -> Option<String> {
    let line = content.lines().find(|l| fstab_subvol(l) == Some(subvol))?;
    let field = line.split_whitespace().nth(3)?;
    let rest: Vec<&str> = field
        .split(',')
        .filter(|o| !o.starts_with("subvol="))
        .collect();
    Some(rest.join(","))
}

/// The referenced-size limit in a `btrfs qgroup show -r -f --raw` report, in bytes.
///
/// The column is found by its header rather than by counting: `btrfs-progs` has added columns
/// across versions, and a hardcoded index would silently read `excl` on the version that did.
/// `none` is btrfs saying there is no limit, and it is not a number — reporting it as one would
/// make a declared `@quota=` compare against a word.
pub fn qgroup_limit_in(output: &str) -> Option<u64> {
    let mut lines = output.lines();
    let column = lines
        .by_ref()
        .find(|l| l.split_whitespace().any(|c| c == "max_rfer"))?
        .split_whitespace()
        .position(|c| c == "max_rfer")?;
    lines
        .filter(|l| !l.trim_start().starts_with("---"))
        .find_map(|l| {
            l.split_whitespace()
                .nth(column)
                .and_then(|v| v.parse().ok())
        })
}

/// `content` with one entry for this subvolume and no other.
///
/// Both matches are dropped and not just one: a declaration that moves a subvolume to a new
/// mount point must leave no entry at the old one, and a declaration that puts a different
/// subvolume at an occupied mount point must not leave two lines fighting over it.
///
/// **Field-wise, never `contains`.** The line this replaced dropped every fstab line with the
/// mount point anywhere in it, so declaring `/mnt` would have deleted `/mnt/data`, `/mnt/home`
/// and any comment that mentioned the path.
pub fn fstab_with(
    content: &str,
    uuid: &str,
    subvol: &str,
    mount_point: &str,
    options: &str,
) -> String {
    let mut lines: Vec<&str> = content
        .lines()
        .filter(|l| fstab_mount_point(l) != Some(mount_point) && fstab_subvol(l) != Some(subvol))
        .collect();
    let entry = format!(
        "UUID={} {} btrfs subvol={},{} 0 0",
        uuid, mount_point, subvol, options
    );
    lines.push(&entry);
    lines.join("\n") + "\n"
}

/// `content` with this subvolume's entry gone, and the mount point it named.
///
/// Removal has to find the entry by subvolume, because a `remove` is handed names and never the
/// options the line carried. Leaving it behind would be worse than untidy: the next boot would
/// try to mount a subvolume that no longer exists, and a machine that fails to mount an fstab
/// entry stops in the initramfs rather than starting.
pub fn fstab_without(content: &str, subvol: &str) -> (String, Option<String>) {
    let point = content
        .lines()
        .find(|l| fstab_subvol(l) == Some(subvol))
        .and_then(fstab_mount_point)
        .map(str::to_string);
    let kept: Vec<&str> = content
        .lines()
        .filter(|l| fstab_subvol(l) != Some(subvol))
        .collect();
    let out = if kept.is_empty() {
        String::new()
    } else {
        kept.join("\n") + "\n"
    };
    (out, point)
}

impl BtrfsBackendCore {
    pub fn new(executor: CommandExecutor) -> Self {
        Self {
            executor,
            name: "btrfs".to_string(),
            mounts_file: std::path::PathBuf::from("/proc/mounts"),
            fstab_file: std::path::PathBuf::from("/etc/fstab"),
        }
    }

    fn btrfs_mounts(&self) -> Vec<BtrfsMount> {
        std::fs::read_to_string(&self.mounts_file)
            .map(|t| btrfs_mounts_in(&t))
            .unwrap_or_default()
    }

    async fn ensure_qgroups(&self, path: &str, sudo: bool) -> Result<()> {
        debug!("BTRFS: Ensuring qgroups are enabled for {}", path);
        self.executor
            .run("btrfs", &["quota", "enable", path], sudo)
            .await?;
        Ok(())
    }

    async fn get_fs_uuid(&self, path: &str) -> Result<String> {
        let output = self
            .executor
            .run_output("btrfs", &["filesystem", "show", path], false)
            .await?;
        fs_uuid_in(&output).ok_or_else(|| {
            Error::Other(format!(
                "could not read the btrfs filesystem UUID for {} — an fstab entry needs it, \
                 and `btrfs filesystem show` did not report one",
                path
            ))
        })
    }

    /// The `subvol=` for a declared path and the filesystem it lives on, refusing rather than
    /// guessing.
    fn subvol_for(&self, path: &str) -> Result<(String, String)> {
        subvol_arg(&self.btrfs_mounts(), path).ok_or_else(|| {
            Error::Validation(format!(
                "`btrfs:{}` is not inside a mounted btrfs filesystem, so there is no subvolume \
                 to mount. Mount the filesystem first, and name a path under it.",
                path
            ))
        })
    }

    /// The fstab's contents, with "no file yet" distinguished from "cannot read it".
    ///
    /// **NotFound is the only error that means empty.** An fstab this process cannot read is
    /// not an empty one: treating any read failure as `""` made `update_fstab` overwrite every
    /// existing entry with just the new line, and made `drop_from_fstab` report "nothing to
    /// drop" and hand the caller a subvolume deletion whose fstab entry survived it — a machine
    /// that stops in the initramfs at the next boot.
    fn read_fstab(&self) -> Result<Option<String>> {
        match fs::read_to_string(&self.fstab_file) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Refused(format!(
                "cannot read {}: {} — refusing to change or drop fstab entries while its \
                 contents are unreadable",
                self.fstab_file.display(),
                e
            ))),
        }
    }

    /// The declared entry, written into fstab so the mount survives a reboot.
    pub fn update_fstab(
        &self,
        uuid: &str,
        subvol: &str,
        mount_point: &str,
        options: &str,
    ) -> Result<()> {
        // A machine with no fstab yet is one whose fstab is empty, not an error: the declaration
        // is the first entry rather than a reason to refuse the mount.
        let content = self.read_fstab()?.unwrap_or_default();
        let updated = fstab_with(&content, uuid, subvol, mount_point, options);
        crate::utils::file::persist(&self.fstab_file, &updated)?;
        Ok(())
    }

    /// Where this subvolume is mounted right now, by the kernel's account — every place, because
    /// btrfs will happily mount one subvolume at several.
    pub fn current_mounts_of(&self, subvol: &str) -> Vec<String> {
        let subvol = subvol.trim_matches('/');
        self.btrfs_mounts()
            .into_iter()
            .filter(|m| m.prefix.trim_matches('/') == subvol)
            .map(|m| m.point)
            .collect()
    }

    /// Drops this subvolume's entry, and says where it had been mounted.
    pub fn drop_from_fstab(&self, subvol: &str) -> Result<Option<String>> {
        let Some(content) = self.read_fstab()? else {
            return Ok(None);
        };
        let (updated, point) = fstab_without(&content, subvol);
        if updated != content {
            crate::utils::file::persist(&self.fstab_file, &updated)?;
        }
        Ok(point)
    }
}

#[async_trait]
impl BackendCore for BtrfsBackendCore {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux") && self.executor.command_exists_sync("btrfs")
    }
    fn probes(&self) -> Vec<String> {
        vec!["btrfs".into()]
    }

    fn needs_root(&self) -> bool {
        // Filesystem level modifications (subvolumes, mounts) require root.
        true
    }
}

#[async_trait]
impl MetadataProvider for BtrfsBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        // Subvolumes are standalone filesystem objects and do not have transitive package deps.
        Ok(vec![])
    }
}

pub struct BtrfsInstallable {
    pub core: Arc<BtrfsBackendCore>,
}

#[async_trait]
impl Installable for BtrfsInstallable {
    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        for spec in specs {
            let path = &spec.name;

            if !Path::new(path).exists() {
                info!("BTRFS: Creating subvolume at {}", path);
                self.core
                    .executor
                    .run("btrfs", &["subvolume", "create", path], sudo)
                    .await?;
            }

            if let Some(quota_size) = spec.options.one("quota") {
                let _ = self.core.ensure_qgroups(path, sudo).await;
                self.core
                    .executor
                    .run("btrfs", &["qgroup", "limit", quota_size, path], sudo)
                    .await?;
            }

            if let Some(mount_point) = spec.options.one("mount") {
                if !Path::new(mount_point).exists() {
                    self.core
                        .executor
                        .run("mkdir", &["-p", mount_point], sudo)
                        .await?;
                }

                let (subvol, fs_point) = self.core.subvol_for(path)?;
                // The filesystem, not the subvolume: `btrfs filesystem show` refuses the latter.
                let uuid = self.core.get_fs_uuid(&fs_point).await?;
                let options = spec
                    .options
                    .one("mount_options")
                    .unwrap_or("defaults")
                    .to_string();

                // Moving a declared mount has to move it. btrfs is happy to hold one subvolume
                // at two places, so writing the new entry and mounting it would leave the old
                // mount live and the machine in a state no declaration describes, until someone
                // rebooted. Best-effort: a busy mount point is a reason to warn, not a reason to
                // fail an install that otherwise worked.
                for stale in self.core.current_mounts_of(&subvol) {
                    if stale.trim_end_matches('/') == mount_point.trim_end_matches('/') {
                        continue;
                    }
                    info!(
                        "BTRFS: releasing {} — {} is declared at {}",
                        stale, path, mount_point
                    );
                    if self
                        .core
                        .executor
                        .run("umount", &[&stale], sudo)
                        .await
                        .is_err()
                    {
                        warn!(
                            "BTRFS: {} is still mounted at {}, which no declaration asks for — \
                             something is using it",
                            path, stale
                        );
                    }
                }

                let core_ref = self.core.clone();
                let mount_str = mount_point.to_string();

                tokio::task::spawn_blocking(move || {
                    core_ref.update_fstab(&uuid, &subvol, &mount_str, &options)
                })
                .await
                .map_err(|e| Error::Other(e.to_string()))??;

                // `mount POINT` reads the entry just written, so the running machine and the
                // next boot mount the same thing — the alternative is a mount that works until
                // someone reboots.
                self.core
                    .executor
                    .run("mount", &[mount_point], sudo)
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
            // The declared mount goes before the subvolume does, in that order: a mounted
            // subvolume cannot be deleted, and an fstab entry outliving its subvolume is a
            // machine that stops in the initramfs at the next boot.
            if let Ok((subvol, _)) = self.core.subvol_for(name) {
                let core_ref = self.core.clone();
                let owned = subvol.clone();
                let point = tokio::task::spawn_blocking(move || core_ref.drop_from_fstab(&owned))
                    .await
                    .map_err(|e| Error::Other(e.to_string()))??;

                // Every current mount holds the subvolume open, and btrfs cannot delete a
                // mounted subvolume. Releasing them all is part of removal, not best-effort
                // tidying: swallowing a failed umount here used to run `subvolume delete`
                // straight into EBUSY *after* the fstab entry was already gone — a half-torn
                // state the failure could have avoided by refusing one step earlier.
                let mut points = self.core.current_mounts_of(&subvol);
                if let Some(point) = point {
                    if !points
                        .iter()
                        .any(|p| p.trim_end_matches('/') == point.trim_end_matches('/'))
                    {
                        points.push(point);
                    }
                }
                for point in points {
                    info!("BTRFS: unmounting {} before deleting {}", point, name);
                    self.core
                        .executor
                        .run("umount", &[&point], sudo)
                        .await
                        .map_err(|e| {
                            Error::Other(format!(
                                "`{}` is still mounted at {} ({}), and btrfs cannot delete a \
                                 mounted subvolume — the deletion is refused rather than left \
                                 half-torn. The fstab entry is already dropped; free whatever \
                                 is using the mount and re-run.",
                                name, point, e
                            ))
                        })?;
                }
            }
            if Path::new(name).exists() {
                info!("BTRFS: Deleting subvolume {}", name);
                self.core
                    .executor
                    .run("btrfs", &["subvolume", "delete", name], sudo)
                    .await?;
            }
        }
        Ok(())
    }
}

pub struct BtrfsQueryable {
    pub core: Arc<BtrfsBackendCore>,
}

#[async_trait]
impl Queryable for BtrfsQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        // One subvolume, one package — keyed by the filesystem it lives on and its path from
        // that filesystem's root, because `@mount` makes the same object reachable by two
        // paths. Declaring `btrfs:/mnt/fs/data @mount=/srv` leaves it listed under `/mnt/fs/data`
        // *and* under `/srv`, and the second name is undeclared: `remove-orphans` would offer
        // to delete the subvolume the user just declared, under its other name.
        let mut best: std::collections::BTreeMap<(String, String), (usize, String, String)> =
            Default::default();
        let mut mounted_at: std::collections::BTreeMap<(String, String), String> =
            Default::default();
        for mount in self.core.btrfs_mounts() {
            let Ok(output) = self
                .core
                .executor
                .run_output("btrfs", &["subvolume", "list", &mount.point], false)
                .await
            else {
                // One filesystem this process cannot read is not "no subvolumes anywhere".
                // The others still answer, and the alternative — a `?` here — is how asking
                // `/` on a machine with an ext4 root made this whole backend report an error.
                continue;
            };
            for (rel, name) in subvolume_paths(&mount.point, &mount.prefix, &output) {
                // A mount whose own `subvol=` IS this subvolume is where it is mounted. That is
                // the second name being collapsed below, read as the fact it actually carries,
                // so `sync` can tell a declared `@mount=` that happened from one that did not.
                if mount.prefix.trim_matches('/') == rel.trim_matches('/') {
                    mounted_at.insert(
                        (mount.device.clone(), rel.clone()),
                        mount.point.trim_end_matches('/').to_string(),
                    );
                }
                let key = (mount.device.clone(), rel);
                // The mount closest to the filesystem root wins — the shortest `subvol=`, not
                // the shortest name. That is the path a declaration was written against:
                // `btrfs:/mnt/fs/data @mount=/srv` mounts the object a second time at `/srv`,
                // and answering `/srv` would make `sync` believe the declared path was absent
                // and re-create it on every run, which is the 2026-07-30 bug from the other
                // side. Ties go to the shortest mount point, then lexicographically, so two
                // runs on an unchanged machine answer the same.
                let rank = (mount.prefix.len(), mount.point.clone(), name);
                best.entry(key)
                    .and_modify(|held| {
                        if rank < *held {
                            *held = rank.clone();
                        }
                    })
                    .or_insert(rank);
            }
        }
        Ok(best
            .into_iter()
            .map(|(key, (_, _, name))| {
                let mut p = Package::new(name, "btrfs");
                if let Some(point) = mounted_at.get(&key) {
                    p.properties.insert("mount".to_string(), point.clone());
                }
                p
            })
            .collect())
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        self.list_installed().await
    }

    /// Through the listing, like `zfs:` and `lvm:` — never `Path::exists`.
    ///
    /// Two things turn on it. A plain directory is not a subvolume, and answering "installed"
    /// for one leaves `sync` believing a declaration it never carried out is satisfied; and the
    /// planner reads *this* answer, not `list_installed`, when it asks whether a spec still
    /// needs doing — so a `Package` built here by hand carries no `mount` property and a
    /// declared `@mount=` that never happened stays invisible for ever.
    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let name = name.trim_end_matches('/');
        let Some(mut p) = self
            .list_installed()
            .await?
            .into_iter()
            .find(|p| p.name.trim_end_matches('/') == name)
        else {
            return Ok(None);
        };
        // The quota and the fstab options are read **here and not in `list_installed`**, because
        // each is a question about one named subvolume and the listing walks every subvolume on
        // every btrfs filesystem. Asking there would fan one `sync` out into a subprocess per
        // subvolume — the shape the planner's own dependency walk is capped at one level to
        // avoid. The planner asks `info`, which is the only caller that needs these.
        if let Ok(out) = self
            .core
            .executor
            .run_output(
                "btrfs",
                &["qgroup", "show", "-r", "-f", "--raw", name],
                false,
            )
            .await
        {
            let limit = qgroup_limit_in(&out).map(|b| b.to_string());
            p.properties.insert(
                "quota".into(),
                limit.unwrap_or_else(|| crate::backends::storage::NO_LIMIT.to_string()),
            );
        }
        if let Ok((subvol, _)) = self.core.subvol_for(name) {
            if let Ok(content) = fs::read_to_string(&self.core.fstab_file) {
                if let Some(opts) = fstab_options(&content, &subvol) {
                    p.properties.insert("mount_options".to_string(), opts);
                }
            }
        }
        Ok(Some(p))
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    _cfg: &crate::config::Config,
) {
    let core = Arc::new(BtrfsBackendCore::new(exec.clone()));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(BtrfsInstallable { core: core.clone() }))
            .with_queryable(Arc::new(BtrfsQueryable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}

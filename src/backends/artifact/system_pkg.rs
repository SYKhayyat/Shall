//! Handing a downloaded `.deb`/`.rpm` to the system package manager (D5).
//!
//! `github:`/`web:` may resolve to a file that installs *itself* into a second package
//! database — a `.deb` to `dpkg`, an `.rpm` to `rpm`. When that happens the installing manager,
//! not Shall, owns the files on disk: Shall records **which** manager installed it, and removal,
//! upgrade and dedup all route back through that record (D5, owner 2026-07-24). This module is
//! the one place the argv for that hand-off is built, so github and web cannot drift on what
//! "hand it to dpkg" means, and so the argv is unit-testable without a live apt/rpm box — which
//! is the whole of what is verified here; the live round-trip is the real machine's job.

use super::format::Format;
use crate::core::{Error, Result};
use std::path::Path;

/// The system installer a given artifact format hands itself to, or `None` for a format Shall
/// unpacks itself (a tarball) or that no manager on the supported platforms consumes as a file.
/// Only `.deb`/`.rpm` are wired: `.msi`/`.exe`/`.pkg` are `is_system_package` shapes too, but
/// their silent-install argv is per-vendor and unruled, so they stay unpacked/deployed, not
/// handed off.
pub fn installer_for(format: Format) -> Option<&'static str> {
    match format {
        Format::Deb => Some("dpkg"),
        Format::Rpm => Some("rpm"),
        _ => None,
    }
}

/// True when this format is one D5 hands to a system installer (as opposed to `is_system_package`,
/// which is the broader "installs into a database" set including the Windows/mac shapes Shall
/// does not drive as a file yet).
pub fn is_handoff_format(format: Format) -> bool {
    installer_for(format).is_some()
}

/// The command whose presence means this machine can consume the handoff file. Same as the
/// installer name today, kept separate so a future split (apt provides dpkg) has one place.
pub fn detect_command(format: Format) -> Option<&'static str> {
    installer_for(format)
}

/// The argv that installs `file` through its system manager. `dpkg -i` upgrades in place, so it
/// doubles as the upgrade path; `rpm` uses `-U` (upgrade-or-install) rather than the ruling's
/// literal `-i`, because `rpm -i` *refuses* an already-installed package and the installing
/// backend is required to own the upgrade — `-U` is the one form that installs a new package and
/// replaces an older one with the same argv. The live exec is deferred (D5); this shape is what
/// is tested.
pub fn install_argv(format: Format, file: &Path) -> Result<Vec<String>> {
    let path = file.to_string_lossy().to_string();
    match format {
        Format::Deb => Ok(vec!["dpkg".into(), "-i".into(), path]),
        Format::Rpm => Ok(vec![
            "rpm".into(),
            "-U".into(),
            "--replacepkgs".into(),
            path,
        ]),
        other => Err(Error::Validation(format!(
            "{} is not a file Shall hands to a system package manager",
            other
        ))),
    }
}

/// The argv that removes a package the recorded installer owns. `dpkg -r` / `rpm -e` take the
/// package *name* inside the file, not the filename — which is why the name is read at install
/// time and carried in the lock (`system_package`).
pub fn remove_argv(installer: &str, package: &str) -> Result<Vec<String>> {
    match installer {
        "dpkg" => Ok(vec!["dpkg".into(), "-r".into(), package.into()]),
        "rpm" => Ok(vec!["rpm".into(), "-e".into(), package.into()]),
        other => Err(Error::Validation(format!(
            "`{}` is not a system installer Shall knows how to remove from",
            other
        ))),
    }
}

/// The argv that reads the package name out of a handoff file, before it is installed. That name
/// is the identity the manager will list it under, so it is what removal and dedup key on. Prints
/// exactly the name on stdout.
pub fn query_name_argv(format: Format, file: &Path) -> Result<Vec<String>> {
    let path = file.to_string_lossy().to_string();
    match format {
        Format::Deb => Ok(vec!["dpkg-deb".into(), "-f".into(), path, "Package".into()]),
        Format::Rpm => Ok(vec![
            "rpm".into(),
            "-qp".into(),
            "--queryformat".into(),
            "%{NAME}".into(),
            path,
        ]),
        other => Err(Error::Validation(format!(
            "{} carries no queryable package name",
            other
        ))),
    }
}

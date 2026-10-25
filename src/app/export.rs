// src/app/export.rs
//
// Reverse of `adopt`/import: emit the managed package set as NATIVE manifests
// (Brewfile, requirements.txt, package.json, Aptfile). This is the "no lock-in" escape
// hatch — you can always leave Shall, or hand a native file to a tool that speaks it.
//
// The per-format renderers are pure (state in, text out) so they are unit-tested without
// touching the filesystem or any backend.

use crate::core::{Error, Result};
use std::path::{Path, PathBuf};

/// A managed package reduced to what the exporters need.
pub type Pkg = (String, String, Option<String>);

/// Snapshot the managed set as `(backend, name, version)`, preferring the live installed
/// version and falling back to the recorded one.
/// Every managed package with the version the manager reports for it, asked concurrently.
///
/// **The one implementation, and it fans out.** This existed twice — here and as
/// `insight::resolve_managed` — and both were a serial `for` loop `await`ing `info()` once per
/// *package*, each of which spawns a child process. `shall --timings` measured the result at
/// **1.0× overlap with one wave per child**: a serial loop wearing a fan-out's clothes, ~9s for
/// `sbom` and ~11s for `export` against 3.6s for `list` doing more work over the same managers.
///
/// Bounded by `max_parallel` for the same reason every other fan-out here is: a machine with
/// hundreds of managed packages would otherwise start hundreds of processes at once.
pub async fn managed_pkgs(
    state: &tokio::sync::Mutex<crate::core::StateRegistry>,
    registry: &crate::backends::BackendRegistry,
    max_parallel: usize,
) -> Vec<Pkg> {
    use futures::stream::StreamExt;

    let recorded: Vec<Pkg> = {
        let state = state.lock().await;
        state
            .managed()
            .map(|p| (p.backend.clone(), p.name.clone(), p.version.clone()))
            .collect()
    };

    // `buffered`, not `buffer_unordered`: the SBOM and every export format are documents whose
    // row order should not depend on which manager answered first.
    futures::stream::iter(recorded)
        .map(|(backend, name, rec)| {
            let queryable = registry
                .get(&backend)
                .and_then(|b| b.as_queryable().cloned());
            async move {
                let version = match queryable {
                    Some(q) => match q.info(&name).await {
                        Ok(Some(p)) => p.version.or(rec),
                        _ => rec,
                    },
                    None => rec,
                };
                (backend, name, version)
            }
        })
        .buffered(max_parallel.max(1))
        .collect()
        .await
}

/// The formats `export` can emit, plus which output filename each conventionally uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Brew,
    Pip,
    Npm,
    Apt,
    /// A NixOS module (`J5`). The odd one out: the other four are the native manifest of ONE
    /// manager, and this one is the system configuration of a whole operating system — so it
    /// takes the `nix:` and `nixos:` sets together, which is what a reader would expect
    /// `shall export --format nix` on a NixOS box to hand them.
    Nix,
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "brew" | "brewfile" => Format::Brew,
            "pip" | "requirements" | "pipx" => Format::Pip,
            "npm" | "node" | "package.json" => Format::Npm,
            "apt" | "aptfile" | "deb" => Format::Apt,
            "nix" | "nixos" => Format::Nix,
            _ => return None,
        })
    }

    pub fn filename(self) -> &'static str {
        match self {
            Format::Brew => "Brewfile",
            Format::Pip => "requirements.txt",
            Format::Npm => "package.json",
            Format::Apt => "Aptfile",
            // The name the `nixos:` backend generates, so an export can be dropped straight into
            // `/etc/nixos` and imported. Two files with one job would be two files to keep in
            // step.
            Format::Nix => crate::backends::nixos::GENERATED,
        }
    }

    pub fn all() -> [Format; 5] {
        [
            Format::Brew,
            Format::Pip,
            Format::Npm,
            Format::Apt,
            Format::Nix,
        ]
    }

    /// True if a backend's packages belong in this format's output.
    fn accepts(self, backend: &str) -> bool {
        match self {
            Format::Brew => backend == "brew",
            Format::Pip => backend == "pip" || backend == "pipx",
            Format::Npm => matches!(backend, "npm" | "pnpm" | "yarn" | "bun"),
            Format::Apt => backend == "apt",
            // Both, deliberately. A package installed through the profile and one written into
            // the system configuration are the same package to somebody asking for "my nix
            // packages as a module", and an export that silently dropped half of them would be
            // the wrong answer quietly.
            Format::Nix => backend == "nix" || backend == "nixos",
        }
    }

    /// Render this format from the managed set. Returns `Ok(None)` when no package applies (so
    /// the caller can skip writing an empty file and say so honestly); the error half is a
    /// renderer refusing its input — a Nix name it cannot render safely — reported rather than
    /// written around.
    pub fn render(self, pkgs: &[Pkg]) -> Result<Option<String>> {
        let mut rows: Vec<&Pkg> = pkgs.iter().filter(|(b, _, _)| self.accepts(b)).collect();
        if rows.is_empty() {
            return Ok(None);
        }
        rows.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(Some(match self {
            Format::Brew => {
                let mut s = String::from("# Generated by `shall export` — Homebrew Bundle\n");
                for (_, name, _) in &rows {
                    s.push_str(&format!("brew \"{}\"\n", name));
                }
                s
            }
            Format::Pip => {
                let mut s = String::from("# Generated by `shall export`\n");
                for (_, name, ver) in &rows {
                    match ver {
                        Some(v) if !v.is_empty() && v != "unknown" => {
                            s.push_str(&format!("{}=={}\n", name, v))
                        }
                        _ => s.push_str(&format!("{}\n", name)),
                    }
                }
                s
            }
            Format::Npm => {
                // A minimal but valid package.json with a dependencies map.
                let deps: serde_json::Map<String, serde_json::Value> = rows
                    .iter()
                    .map(|(_, name, ver)| {
                        let spec = match ver {
                            Some(v) if !v.is_empty() && v != "unknown" => format!("^{}", v),
                            _ => "*".to_string(),
                        };
                        (name.clone(), serde_json::Value::String(spec))
                    })
                    .collect();
                let doc = serde_json::json!({
                    "name": "shall-export",
                    "private": true,
                    "dependencies": deps,
                });
                serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"
            }
            Format::Apt => {
                let mut s = String::from("# Generated by `shall export`\n");
                for (_, name, _) in &rows {
                    s.push_str(&format!("{}\n", name));
                }
                s
            }
            // **Rendered by the backend, not here.** `backends::nixos::render` is what the
            // `nixos:` prefix writes into `/etc/nixos`, and it is the text
            // `scripts/nix-validate.sh` points a real Nix parser at. A second renderer for the
            // same file format would be a second thing to keep valid, and only one of the two
            // would be under that gate.
            //
            // Versions are dropped on purpose: a NixOS module pins packages by pinning nixpkgs,
            // not per package, which is the same reason `nixos:` refuses an `@version=`.
            //
            // Packages only, and that is not an omission: `export` renders what the *managed
            // set* holds, and a `service:` or `firewall:` line is not a package in it. The
            // services and the perimeter reach the same file through `sync` on a NixOS host.
            Format::Nix => crate::backends::nixos::render(&crate::backends::nixos::Module {
                packages: rows.iter().map(|(_, name, _)| name.clone()).collect(),
                ..Default::default()
            })?,
        }))
    }
}

/// What `export` did, or would do, with one format's file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// No package maps to this format, so no file.
    NoPackages,
    Wrote(PathBuf),
    /// The natural filename was taken by a file Shall did not write, so the export landed
    /// beside it under `renamed`.
    WroteBeside {
        taken: PathBuf,
        renamed: PathBuf,
    },
    /// `--dry-run`: this is where the bytes would have gone.
    WouldWrite(PathBuf),
}

/// The export's filename with `.shall` inserted before the extension, used when the real
/// filename is already taken: `package.json` -> `package.shall.json`, `Brewfile` ->
/// `Brewfile.shall`.
pub fn beside(name: &str) -> String {
    match name.split_once('.') {
        Some((stem, ext)) => format!("{}.shall.{}", stem, ext),
        None => format!("{}.shall", name),
    }
}

/// The first free name in the `beside` family, so a second export never clobbers the first
/// export's fallback either.
pub async fn free_path(out_dir: &Path, name: &str) -> PathBuf {
    let base = beside(name);
    let first = out_dir.join(&base);
    if !tokio::fs::try_exists(&first).await.unwrap_or(false) {
        return first;
    }

    // One directory read rather than up to 998 `stat` calls and 998 `format!` allocations to
    // answer a question the directory listing answers once.
    let mut taken: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut entries) = tokio::fs::read_dir(out_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            taken.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    for n in 2..1000 {
        let candidate = format!("{}.{}", base, n);
        if !taken.contains(&candidate) {
            return out_dir.join(candidate);
        }
    }
    first
}

/// Emit one or all formats. When `to_stdout`, a single format is printed; otherwise files are
/// written into `out_dir`.
///
/// `export` runs in directories full of real files it did not create — the default `out_dir`
/// is `.`, and `package.json`/`Brewfile` are exactly the names a real project already uses.
/// So an existing file is never replaced unless `force` says to: the export goes to a
/// non-colliding name instead, and the caller reports where.
pub async fn export(
    state: &tokio::sync::Mutex<crate::core::StateRegistry>,
    registry: &crate::backends::BackendRegistry,
    config: &crate::config::Config,
    format: Option<Format>,
    out_dir: &Path,
    to_stdout: bool,
    force: bool,
) -> Result<Vec<(String, Outcome)>> {
    // `dry_run` and `max_parallel` arrived as two separate arguments and took this past the
    // seven clippy allows. They are both readings of one thing the caller already holds, so the
    // fix is the configuration itself rather than a wider signature.
    let dry_run = config.dry_run;
    let pkgs = managed_pkgs(state, registry, config.max_parallel).await;
    let formats: Vec<Format> = match format {
        Some(f) => vec![f],
        None => Format::all().to_vec(),
    };

    let mut results = Vec::new();
    if to_stdout {
        // Only meaningful for a single format; print it (or a note if empty).
        let f = formats[0];
        match f.render(&pkgs)? {
            Some(text) => print!("{}", text),
            None => eprintln!("# no packages map to {}", f.filename()),
        }
        return Ok(results);
    }

    if !dry_run {
        crate::utils::file::ensure_dir_async(out_dir).await?;
    }
    for f in formats {
        let name = f.filename().to_string();
        let text = match f.render(&pkgs)? {
            Some(t) => t,
            None => {
                results.push((name, Outcome::NoPackages));
                continue;
            }
        };
        let natural = out_dir.join(&name);
        let taken = tokio::fs::try_exists(&natural).await.unwrap_or(false);

        let (target, outcome) = if taken && !force {
            let alt = free_path(out_dir, &name).await;
            let o = if dry_run {
                Outcome::WouldWrite(alt.clone())
            } else {
                Outcome::WroteBeside {
                    taken: natural.clone(),
                    renamed: alt.clone(),
                }
            };
            (alt, o)
        } else if dry_run {
            (natural.clone(), Outcome::WouldWrite(natural.clone()))
        } else {
            (natural.clone(), Outcome::Wrote(natural.clone()))
        };

        if !dry_run {
            tokio::fs::write(&target, text).await.map_err(Error::from)?;
        }
        results.push((name, outcome));
    }
    Ok(results)
}

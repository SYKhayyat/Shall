//! **The four files that decide what a user downloads must name the same targets.**
//!
//! `#21` names this as the release invariant most exposed to drift, and it is exposed because
//! the agreement is invisible from every one of the four places on its own:
//!
//! - `ci.yml`'s build matrix decides what gets compiled;
//! - `ci.yml`'s release job decides what the assets are *called* (`dist/shall-<target>`, and
//!   `.exe` for Windows);
//! - `scripts/install.sh` decides what a Linux or macOS user *asks for*;
//! - `scripts/install.ps1` decides what a Windows user asks for.
//!
//! Each is individually reasonable. A target added to the matrix and not to the release job
//! builds a binary nobody publishes; a target added to the release job and not to an installer
//! publishes a binary nobody can download from the documented line; a target renamed in one place
//! and not another publishes an asset the installer's URL does not resolve. None of those
//! produces a compile error, a test failure, or a red CI run before the tag — and the tag is the
//! one push nobody rehearses. So the check is here, where every push runs it.
//!
//! **This reads files and asserts a relationship**, so it is a source-level control and says so.
//! What makes it worth having is that it is the relationship and not a copy of any one file's
//! text: each list is *parsed* out of its own file's own shape, and the assertion is set
//! equality in the directions that matter. The one thing it cannot check is that a published
//! release actually carries the assets — that needs a tag, and this test prints why it did not
//! try rather than pretending.
//!
//! The acceptance criterion in `#21` asks for executable tests over source scans. This is the
//! executable half for the release path: it fails today if any of the four files is edited to
//! disagree with the others, which is the only way the invariant has ever broken here.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel))
        .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
        .replace("\r\n", "\n")
}

/// The build matrix's targets, read from the `target:` keys of its `include:` rows.
fn matrix_targets(ci: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in ci.lines() {
        if let Some(i) = line.find("target: ") {
            let name = line[i + "target: ".len()..].trim();
            // A row value is bare; anything else here means the shape changed and this scan
            // would silently match nothing.
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            {
                out.insert(name.to_string());
            }
        }
    }
    out
}

/// The targets the release job's own assertion names, which is the list it requires on disk.
fn release_asserted_targets(ci: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let marker = "The binaries the release would publish are present";
    let at = ci
        .find(marker)
        .unwrap_or_else(|| panic!("ci.yml must retain the step `{marker}`"));
    // The step's own shell block: from the marker to the next `- name:` at the same indent.
    let block = &ci[at..];
    let end = block.find("\n    - name:").unwrap_or(block.len());
    let block = &block[..end];

    // Two shapes in that block: a `for target in …` list, and a `test -f` / `missing=` line per
    // odd-one-out target. Both are read, because the Windows row is deliberately the second.
    for line in block.lines() {
        if let Some(i) = line.find("for target in ") {
            // The list ends at the shell keyword `do`, which is the next word of the same line,
            // and its last name carries the `;` that separates the two.
            for t in line[i + "for target in ".len()..].split_whitespace() {
                let t = t.trim_end_matches(';');
                if t == "do" || t.ends_with('-') || t.is_empty() {
                    break;
                }
                if t.contains('-')
                    && t.chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                {
                    out.insert(t.to_string());
                }
            }
        }
        // `dist/shall-<target>` and `dist/shall-<target>.exe`. A target always carries a
        // hyphen (every triple does), which is what keeps `${target}` out: that is the loop
        // variable, not a name, and reading it as one invented a target called `do`.
        let mut rest = line;
        while let Some(i) = rest.find("dist/shall-") {
            rest = &rest[i + "dist/shall-".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
                .collect();
            let name = name.strip_suffix(".exe").unwrap_or(&name).to_string();
            // `${target}` scans as `$`, which the take_while stops on, so an empty or
            // hyphen-free read is the loop variable and not an asset.
            if name.contains('-') {
                out.insert(name);
            }
        }
    }
    out
}

/// Every triple the Unix installer can ask for, read from its `echo <triple>` arms.
fn installer_triples(install: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let at = install
        .find("target_triple() {")
        .expect("install.sh must keep a `target_triple` function; this scan keys on it");
    let block = &install[at..];
    let end = block.find("\n}\n").unwrap_or(block.len());
    for line in block[..end].lines() {
        // `x86_64)        echo x86_64-unknown-linux-musl ;;`
        if let Some(i) = line.find("echo ") {
            let name: String = line[i + "echo ".len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                .collect();
            if name.contains('-') {
                out.insert(name);
            }
        }
    }
    out
}

/// The target `install.ps1` asks for, read from the asset name it downloads.
///
/// The literal is the whole asset name (`shall-<target>.exe`) because that is what the URL needs;
/// the published set holds bare targets, so the two shapes are reconciled here rather than in
/// three assertions.
fn powershell_asset(ps1: &str) -> String {
    let i = ps1
        .find("$asset = '")
        .expect("install.ps1 must keep `$asset = '<name>'`; this scan keys on it");
    let rest = &ps1[i + "$asset = '".len()..];
    let end = rest.find('\'').expect("unterminated `$asset` literal");
    let asset = &rest[..end];
    let asset = asset.strip_suffix(".exe").unwrap_or(asset);
    asset
        .strip_prefix("shall-")
        .unwrap_or_else(|| panic!("install.ps1's asset `{asset}` must be named `shall-<target>`"))
        .to_string()
}

/// **Nothing is built that is not published, and nothing is published that is not built.**
#[test]
fn the_release_publishes_exactly_the_targets_the_matrix_builds() {
    let ci = read(".github/workflows/ci.yml");
    let built = matrix_targets(&ci);
    let published = release_asserted_targets(&ci);

    assert!(
        built.len() >= 7,
        "the build matrix scan found {} target(s) ({built:?}); the release path has carried seven \
         for long enough that a smaller number means this scan stopped matching ci.yml.",
        built.len()
    );

    let built_not_published: Vec<&String> = built.difference(&published).collect();
    assert!(
        built_not_published.is_empty(),
        "these targets are built and never published, so the release silently has no binary for \
         them: {built_not_published:?}"
    );

    let published_not_built: Vec<&String> = published.difference(&built).collect();
    assert!(
        published_not_built.is_empty(),
        "the release requires assets for targets the matrix does not build, so the release job \
         fails on the push that matters — a tag: {published_not_built:?}"
    );
}

/// **Every target an installer asks for is one the release publishes**, and every published
/// target is reachable from a documented install line.
///
/// Both directions, because the two failures are different: asking for an unpublished asset
/// means the documented install line 404s and silently falls back to a source build; publishing
/// one nobody asks for means a platform was built and cannot be installed.
#[test]
fn every_installer_asks_for_a_target_the_release_publishes() {
    let ci = read(".github/workflows/ci.yml");
    let published = release_asserted_targets(&ci);
    let sh = installer_triples(&read("scripts/install.sh"));
    let ps1 = powershell_asset(&read("scripts/install.ps1"));

    assert!(
        !sh.is_empty(),
        "the install.sh scan found no triples, so it has stopped matching that file and would \
         pass by finding nothing"
    );
    for t in &sh {
        assert!(
            published.contains(t),
            "`scripts/install.sh` asks for `shall-{t}`, which the release does not publish — the \
             documented install line 404s on that platform and falls back to a source build \
             without saying so. Published: {published:?}"
        );
    }

    assert!(
        published.contains(&ps1),
        "`scripts/install.ps1` asks for `shall-{ps1}.exe`, which the release does not publish. \
         Published: {published:?}"
    );

    // The other direction, with the two gnu rows deliberately exempt: `install.sh`'s own header
    // says `-gnu` is published for both architectures "for anyone who wants it", so they are the
    // documented case of a published-and-not-defaulted target.
    let reachable: BTreeSet<String> = sh.iter().cloned().chain(std::iter::once(ps1)).collect();
    let unreachable: Vec<&String> = published
        .iter()
        .filter(|t| !reachable.contains(*t) && !t.contains("-linux-gnu"))
        .collect();
    assert!(
        unreachable.is_empty(),
        "these targets are published and no installer asks for them, so a platform is built and \
         cannot be installed from any documented line: {unreachable:?}"
    );
}

/// **The asset name the release job writes is the name the installers build.**
///
/// Both sides are already checked against the published set above, but they are checked through
/// two different readings — `dist/shall-<target>` in one file, `$REPO/releases/download/…/shall-$triple`
/// and `shall-x86_64-pc-windows-msvc.exe` in the others. A change to *either* naming shape would
/// leave the set comparison passing while the URL stopped resolving, so the shape itself is
/// pinned once, here: the release writes `shall-<target>`, and both installers ask for
/// `shall-<target>`.
#[test]
fn the_asset_name_shape_is_the_same_in_the_release_job_and_both_installers() {
    let ci = read(".github/workflows/ci.yml");
    assert!(
        ci.contains(r#"cp "${d}shall.exe" "dist/shall-${target}.exe""#)
            && ci.contains(r#"cp "${d}shall" "dist/shall-${target}""#),
        "the release job must keep naming assets `shall-<target>` (and `.exe` on Windows); the \
         installers build their URLs from that shape"
    );

    let sh = read("scripts/install.sh");
    assert!(
        sh.contains("shall-$triple"),
        "install.sh must keep building the asset name as `shall-$triple`, which is the shape the \
         release job writes"
    );

    let ps1 = read("scripts/install.ps1");
    assert!(
        ps1.contains("'shall-x86_64-pc-windows-msvc.exe'"),
        "install.ps1 must keep asking for the Windows asset by its full published name"
    );
}

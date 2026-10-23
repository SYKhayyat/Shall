// Each parser takes the raw stdout plus the backend id, so a single implementation can be
// reused by several managers whose output shares a shape. They are wired to backends via
// non-capturing closures in `backends/registry.rs`, e.g.
// `|o| ecosystem::ws_name_version(o, "guix")`.
//
// Lenient about *lines*, strict about the *answer*. Package-manager output drifts across
// versions, so a parser skips blank lines, table headers and decorative rows rather than
// erroring on each. What it may not do is turn a whole output it recognised nothing in into an
// empty list: that is a different fact, the planner acts on it in the opposite direction, and
// `or_unrecognised` is where the two are told apart. See `parsers::Unrecognised`.

use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult, Unrecognised};
use crate::utils::text::sanitize;

/// Header tokens that commonly lead a table's first column and must not be mistaken for a
/// package name.
///
/// **An explicit list, not an all-caps guess.** The old second arm deleted ANY all-caps word
/// of two or more letters, and Hackage ships packages literally named `HTTP`, `GLUT` and
/// `ALUT` — every listing dropped them as headers and every sync reinstalled them for ever.
/// Unknown localized headers are the row-level check in [`is_noise_line`]'s doc, which sees
/// the whole line before deciding.
fn is_header_token(tok: &str) -> bool {
    matches!(
        tok,
        "NAME"
            | "Name"
            | "PLUGIN"
            | "Plugin"
            | "Package"
            | "PACKAGE"
            | "Repository"
            | "Bucket"
            | "Source"
            | "Version"
            | "VERSION"
            | "Global"
    )
}

/// True for a decorative / non-data line: empty, a tree connector, a dashed separator, a
/// bracketed status banner, or a table header.
///
/// **The header belongs here and not in each reader's `filter_map`.** Ten readers dropped a
/// header row from the *packages* while leaving it in the *candidates*, and `or_unrecognised`
/// reads a non-empty candidate list that yielded nothing as *"I could not understand this"*. So
/// `helm plugin list` on a cluster with no plugins — which prints exactly
/// `NAME\tVERSION\tDESCRIPTION` and nothing else — was refused as unreadable rather than
/// answered as empty, and every verb that needs the installed set refused with it.
///
/// **The row-level header test is structural, not lexical.** A line whose first token is
/// ALL-CAPS reads as a header only when no other column opens with a digit: header rows carry
/// column labels (`NAME VERSION`), data rows carry versions (`HTTP 2000.0.8`, `GLUT 2.7.0`),
/// and a version starts with a digit in every ecosystem this family reads. That one shape test
/// separates the two without knowing any manager's language.
fn is_noise_line(line: &str) -> bool {
    let t = line.trim();
    let mut toks = t.split_whitespace();
    let caps_header = match (toks.next(), toks.next()) {
        (Some(first), Some(second))
            if first.len() >= 2 && first.chars().all(|c| c.is_ascii_uppercase()) =>
        {
            // No digit-leading column anywhere in the rest of the row → labels, not data.
            !t.split_whitespace()
                .skip(1)
                .any(|c| c.starts_with(|ch: char| ch.is_ascii_digit()))
                && second
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
        }
        _ => false,
    };
    t.is_empty()
        || t.split_whitespace().next().is_some_and(is_header_token)
        || caps_header
        || t.starts_with('#')
        || t.chars()
            .all(|c| matches!(c, '-' | '=' | '─' | '│' | '├' | '└' | ' '))
        || t.starts_with(['─', '│', '├', '└'])
        || is_placeholder(t)
        || is_empty_result_sentence(t)
        // A heading that introduces the list — `nimble list --installed` prints
        // `Package list format:` above its legend when nothing is installed, and that is a
        // correct empty answer rather than a listing nobody could read.
        || crate::parsers::is_prose_line(t)
}

/// `{PackageName}`, `<name>` — a manager describing the shape of its own output. `nimble list
/// --installed` prints its format legend when nothing is installed, and its first token became
/// a package called `{PackageName}` (the S22 failure, in a second manager). No real package
/// name opens with a placeholder bracket.
fn is_placeholder(line: &str) -> bool {
    let first = line.split_whitespace().next().unwrap_or("");
    first.starts_with('{') || first.starts_with('<')
}

/// "No global environments found." — a manager saying it has nothing, not a package called
/// `No`. Every parser here takes the first token of a line, so an unfiltered empty-result
/// banner becomes a phantom package that `adopt` would write into a manifest and
/// `purge-undeclared` would try to delete.
///
/// Prose, not identifiers: a package line is tokens, never a sentence ending in a period.
fn is_empty_result_sentence(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("no ") && lower.ends_with('.') && line.split_whitespace().count() > 2
}

/// One package name per line, taking the first whitespace token. For managers whose list/
/// search prints bare identifiers (opam `--short`, spack `list`, emerge `qlist -I` atoms).
/// Skips blank lines and table headers.
///
/// **Not pixi.** This said "pixi `search`" and was wrong: pixi prints a detail record, and the
/// first token of each of its lines is a field label, a separator or a bare version number —
/// 20 of them in one search, measured. `pixi_search` reads the record. The comment naming a
/// manager this function does not suit is what kept the routing wrong, so the list here is the
/// list that has been checked against that manager's real output and nothing else.
pub fn names_only(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let tok = l.split_whitespace().next()?;
            if is_header_token(tok) {
                return None;
            }
            Some(Package::new(tok, backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// `name version [extra…]` per line, whitespace- or tab-separated. Covers cabal
/// (`--simple-output`), spack (`find --format "{name} {version}"`), pub (`global list`),
/// krew (`list`), helm (`plugin list`), guix (`package -I`), luarocks (`--porcelain`).
/// The second column is treated as the version; any trailing columns are ignored.
pub fn ws_name_version(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let name = parts.next()?;
            if is_header_token(name) {
                return None;
            }
            match parts.next() {
                Some(ver) => Some(Package::with_version(name, ver, backend)),
                None => Some(Package::new(name, backend)),
            }
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// `cabal list --installed --simple-output`, which is `ws_name_version` plus the chatter cabal
/// writes to **stdout** when it has no config file yet.
///
/// ```text
/// Config file path source is default config file.
/// Config file not found: /root/.config/cabal/config
/// Writing default configuration to /root/.config/cabal/config
/// Cabal 3.10.3.0
/// ```
///
/// Read as `name version` those three lines are the packages `Config@file`, `Config@file` and
/// `Writing@default`, and the first two collide on a name — so a container's first `cabal list`
/// reported three packages nobody installed, ahead of the thirty-eight that were real.
///
/// Haskell versions are PVP: dot-separated integers, always. A second column that does not start
/// with a digit is therefore not a version, and the line it is on is not a package.
pub fn cabal_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let name = parts.next()?;
            if is_header_token(name) {
                return None;
            }
            let ver = parts.next()?;
            ver.starts_with(|c: char| c.is_ascii_digit())
                .then(|| Package::with_version(name, ver, backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// `uv tool list`, which is one `name vVERSION` line per tool followed by one `- executable`
/// line per command that tool exposes.
///
/// ```text
/// ruff v0.16.2
/// - ruff
/// ```
///
/// Read as `ws_name_version` the second line is a package named `-` at version `ruff`, and every
/// uv machine reports twice as many packages as it has. The `v` goes too: uv prints it, nobody
/// writes it, and `uv:ruff@0.16.2` would never match a recorded `v0.16.2`.
pub fn uv_tool_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean
        .lines()
        .filter(|l| !is_noise_line(l) && !l.trim_start().starts_with("- "))
        .collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let name = parts.next()?;
            if is_header_token(name) {
                return None;
            }
            match parts.next() {
                Some(ver) => Some(Package::with_version(
                    name,
                    ver.strip_prefix('v').unwrap_or(ver),
                    backend,
                )),
                None => Some(Package::new(name, backend)),
            }
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// eopkg `list-installed` / `search`: `name - Short description`. Take the field before
/// the first ` - ` (falling back to the first token).
pub fn eopkg_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let name = l.split(" - ").next().unwrap_or(l).trim();
            let name = name.split_whitespace().next().unwrap_or(name);
            if name.is_empty() || is_header_token(name) {
                return None;
            }
            Some(Package::new(name, backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// guix `search`: recutils output with `name: <pkg>` and `version: <ver>` fields, one
/// blank-line-separated record per package. Pair each `name:` with the following
/// `version:` in the same record.
pub fn guix_search(output: &str, backend: &str) -> Vec<Package> {
    let clean = sanitize(output);
    let mut out = Vec::new();
    let mut pending_name: Option<String> = None;
    for line in clean.lines() {
        if let Some(rest) = line.strip_prefix("name:") {
            pending_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("version:") {
            if let Some(name) = pending_name.take() {
                out.push(Package::with_version(&name, rest.trim(), backend));
            }
        } else if line.trim().is_empty() {
            // record boundary: a name with no version still counts.
            if let Some(name) = pending_name.take() {
                out.push(Package::new(name, backend));
            }
        }
    }
    if let Some(name) = pending_name.take() {
        out.push(Package::new(name, backend));
    }
    out
}

/// Strip a Slackware package filename (`name-version-arch-build`) down to its name. The
/// last three `-`-separated fields are version, arch and build; everything before them is
/// the (possibly hyphenated) name.
fn slack_pkgname(field: &str) -> &str {
    let parts: Vec<&str> = field.split('-').collect();
    if parts.len() >= 4 {
        // Rejoin all but the last three fields; find the byte index of the 3rd-from-last '-'.
        let keep = parts.len() - 3;
        let mut idx = 0;
        let mut seen = 0;
        for (i, c) in field.char_indices() {
            if c == '-' {
                seen += 1;
                if seen == keep {
                    idx = i;
                    break;
                }
            }
        }
        if idx > 0 {
            return &field[..idx];
        }
    }
    field
}

/// slackpkg installed list: output of `ls /var/log/packages`, one `name-ver-arch-build`
/// filename per line.
pub fn slackpkg_installed(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let found = candidates
        .iter()
        .map(|l| Package::new(slack_pkgname(l), backend))
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// slackpkg `search`: rows like `[ installed ] - name-ver-arch-build`. Pull the package
/// field after `] - ` (or `- `) and strip it to a name.
pub fn slackpkg_search(output: &str, backend: &str) -> Vec<Package> {
    let clean = sanitize(output);
    clean
        .lines()
        .filter_map(|l| {
            let field = l.rsplit("- ").next()?.trim();
            if field.is_empty() || !field.contains('-') {
                return None;
            }
            Some(Package::new(slack_pkgname(field), backend))
        })
        .collect()
}

/// nimble `list --installed`. Name is the first token; the version is the first entry inside
/// the brackets, in either of the two layouts nimble has shipped.
///
/// Nimble 0.13 printed a bare version list, `pkgname  [1.0.0, 0.9.0]`. Nimble 2 prints a record
/// per version instead:
///
/// ```text
/// nimpy  [(version: 0.2.1, checksum: 22173fb24ce9ca9d1c1db63fe15bdfb14e69c76a)]
/// ```
///
/// Taking the first comma-separated field of that yields `(version: 0.2.1`, which is what this
/// build recorded as nimpy's version until a container was asked. A pin can never match it and
/// `shall list` prints it verbatim.
pub fn nimble_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let t = l.trim();
            let name = t.split_whitespace().next()?;
            if is_header_token(name) {
                return None;
            }
            if let (Some(open), Some(close)) = (t.find('['), t.find(']')) {
                if close > open + 1 {
                    let first = t[open + 1..close].split(',').next().unwrap_or("");
                    // `(version: 0.2.1` in the record layout, `1.0.0` in the bare one. The
                    // label is what differs; the value is after the last colon either way.
                    let ver = first
                        .rsplit(':')
                        .next()
                        .unwrap_or(first)
                        .trim()
                        .trim_start_matches('(')
                        .trim();
                    if !ver.is_empty() {
                        return Some(Package::with_version(name, ver, backend));
                    }
                }
            }
            Some(Package::new(name, backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// mix `archive`: lines like `* hex-2.0.6`. Strip the leading bullet, then split the
/// trailing `-version` off the archive name.
pub fn mix_archive(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates: Vec<&str> = clean.lines().filter(|l| !is_noise_line(l)).collect();
    let found = candidates
        .iter()
        .filter_map(|l| {
            let t = l.trim();
            let entry = t.strip_prefix("* ").or_else(|| t.strip_prefix('*'))?.trim();
            if entry.is_empty() {
                return None;
            }
            match entry.rsplit_once('-') {
                Some((name, ver))
                    if !name.is_empty()
                        && ver.chars().next().is_some_and(|c| c.is_ascii_digit()) =>
                {
                    Some(Package::with_version(name, ver, backend))
                }
                _ => Some(Package::new(entry, backend)),
            }
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// asdf `list`: non-indented lines are plugin (tool) names; indented lines are installed
/// versions of the preceding plugin and are skipped.
/// **This parser's own judgement about not understanding**, which is not the shared one. A
/// plugin added with nothing installed under it — `jq` followed by `  No versions installed` —
/// is a real and common state, and it produces zero packages out of two data lines. The shared
/// rule would call that drift on every machine that added a plugin and did not install a
/// version yet. So the count here is of lines that fit *neither* shape the format has, and an
/// output made entirely of understood lines is an empty answer however many of them there are.
pub fn asdf_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let mut out: Vec<Package> = Vec::new();
    let mut plugin: Option<String> = None;
    let mut unaccounted: Vec<&str> = Vec::new();

    // A plugin line is unindented; the versions installed under it are indented beneath it. A
    // plugin with none still prints — `jq` followed by `  No versions installed` — so the name
    // alone means "added", not "installed", and reading it as installed is drift `sync` can
    // never converge: it removes the version and finds it again on the next run.
    for line in clean.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let version = line.trim();
            if version.eq_ignore_ascii_case("No versions installed") {
                continue;
            }
            match plugin.take() {
                Some(name) => out.push(Package::new(&name, backend)),
                // An indented version with no plugin above it: the indentation that carries this
                // format's whole meaning did not hold.
                None => unaccounted.push(line),
            }
            continue;
        }
        let name = line.trim();
        if name.starts_with('*') || is_header_token(name) {
            plugin = None;
            continue;
        }
        plugin = Some(name.to_string());
    }
    or_unrecognised(backend, out, &unaccounted)
}

/// emerge `--search`: package hits are `*  category/pkg` lines.
pub fn emerge_search(output: &str, backend: &str) -> Vec<Package> {
    let clean = sanitize(output);
    clean
        .lines()
        .filter_map(|l| {
            let t = l.trim_start();
            let atom = t.strip_prefix("* ")?.trim();
            if atom.is_empty() || !atom.contains('/') {
                return None;
            }
            Some(Package::new(atom, backend))
        })
        .collect()
}

/// pixi `global list`: tree rows like `├── python: 3.11.0` (older `- python 3.11.0`).
///
/// **Only the top level is a package.** pixi prints each environment's properties as indented
/// children of its row:
///
/// ```text
/// └── ripgrep: 15.2.0
///     └─ exposes: rg
/// ```
///
/// and `exposes` is the word for *this environment puts these binaries on PATH*, not a second
/// tool. Reported as one, it became a package `list` printed and `check` counted, with `rg` as
/// its version. The depth is the tool's own structure, so this reads that rather than
/// blocklisting the property names pixi happens to print today.
///
/// **Every unindented line must resolve to one of three things**: a package, pixi's own banner
/// (a multi-word left side before the colon, which is what
/// `Global environments as specified in 'C:\…'` is), or noise it already names. **This parser has
/// no unread case, and saying so is the honest answer** rather than inventing one. The failure mode
/// this parser has ever had is *junk*, not emptiness — `exposes: rg` reported as a tool — and
/// there are captured fixtures for that. Forcing a made-up emptiness rule on it would redden a
/// machine with pixi installed and nothing in it, which is the one case its banner covers.
///
/// **And a line that resolves to NONE of those is an unread answer, said out loud.** This was
/// the one reader in the family ending `Ok(found)` unconditionally — correct today, and
/// structurally incapable of noticing the day pixi changes its format, because every byte of
/// junk silently became nothing. The three-way classification makes the fourth case visible:
/// junk is now a refusal naming its sample, not an empty listing wearing one.
pub fn pixi_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    // Only the unindented rows are packages; the indented ones are an environment's properties.
    //
    // The noise check belongs *after* the tree connectors are trimmed: a row reads
    // `└── ripgrep: 15.2.0`, so testing the raw line would drop every package pixi printed and
    // leave only the banner.
    let mut found = Vec::new();
    let mut unresolved: Option<&str> = None;
    for l in clean
        .lines()
        .filter(|l| !l.starts_with(char::is_whitespace))
    {
        let t = l
            .trim()
            .trim_start_matches(|c: char| {
                c.is_whitespace() || matches!(c, '├' | '└' | '│' | '─' | '-' | '*')
            })
            .trim();
        if t.is_empty() || is_noise_line(t) {
            continue;
        }
        if let Some((left, right)) = t.split_once(':') {
            let name = left.trim();
            // A real package name is a single token; a multi-word left side is a banner
            // like "Global environments at /path:" and must be skipped.
            if name.is_empty() || name.contains(char::is_whitespace) || is_header_token(name) {
                continue;
            }
            let ver = right.trim();
            if ver.is_empty() {
                found.push(Package::new(name, backend));
            } else {
                found.push(Package::with_version(name, ver, backend));
            }
            continue;
        }
        let mut parts = t.split_whitespace();
        match (parts.next(), parts.next(), parts.next()) {
            // One bare token: a package this pixi prints without a version.
            (Some(name), None, _) if !is_header_token(name) => {
                found.push(Package::new(name, backend));
            }
            // `name 3.11.0` — the older shape, whose second column is a version.
            (Some(name), Some(ver), rest)
                if !is_header_token(name)
                    && ver.starts_with(|c: char| c.is_ascii_digit())
                    && rest.is_none() =>
            {
                found.push(Package::with_version(name, ver, backend));
            }
            _ => {
                // Not a package, not a banner, not noise: the fourth thing this parser used
                // to swallow. Said, once, with the line that said it.
                unresolved.get_or_insert(t);
            }
        }
    }
    if let Some(sample) = unresolved {
        return Err(Unrecognised {
            backend: backend.to_string(),
            data_lines: clean
                .lines()
                .filter(|l| !l.starts_with(char::is_whitespace) && !l.trim().is_empty())
                .count(),
            sample: sample.to_string(),
        });
    }
    Ok(found)
}

/// Parse `pixi search`, which prints a **detail record** rather than a list of names.
///
/// It was routed to [`names_only`], documented there as "search prints bare identifiers". Real
/// pixi output is `Name`/`Version`/`Build`/`Size`/`License`/… field rows plus a build table, so
/// taking the first token of each line produced 19 junk rows in one search: the field labels
/// themselves, the `-----` separator, `...`, and bare version numbers from the "Other Versions"
/// table. The record carries the answer in its `Name` and `Version` fields; this reads those.
pub fn pixi_search(output: &str, backend: &str) -> Vec<Package> {
    let clean = sanitize(output);
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for line in clean.lines() {
        let mut fields = line.split_whitespace();
        match (fields.next(), fields.next()) {
            (Some("Name"), Some(v)) if name.is_none() => name = Some(v.to_string()),
            (Some("Version"), Some(v)) if version.is_none() => version = Some(v.to_string()),
            _ => {}
        }
        if name.is_some() && version.is_some() {
            break;
        }
    }
    match name {
        Some(n) => {
            let mut p = Package::new(n, backend);
            p.version = version;
            vec![p]
        }
        // "No packages found matching 'x'" is an answer, and an empty one.
        None => vec![],
    }
}

/// pixi `global list --json`: an array of environments, each with its dependencies (`Q43`).
///
/// **The environment is the package, and its version comes from the dependency of the same
/// name.** That is what the text form prints as `ripgrep: 15.2.0`, and this must not widen it:
/// an environment may pull in dependencies nobody declared, and reporting those as installed
/// packages is the fault [`pixi_list`] already had to fix once, where `exposes: rg` became a
/// package named `exposes` at version `rg`.
///
/// **Both early returns used to be `Vec::new()`**, which is this whole finding in two lines:
/// output that is not JSON, and JSON that is not an array, are two ways of not understanding
/// the answer, and both were spelled *"the machine has nothing installed"*. This is also the
/// negotiated `--json` path, so the version of pixi that does not have the flag is precisely
/// the one whose usage message would have been read as an empty machine.
pub fn pixi_list_json(output: &str, backend: &str) -> ParseResult {
    let Some(doc) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json(backend, vec![], None, "not JSON", output);
    };
    let envs = doc.as_array();
    let found = envs
        .into_iter()
        .flatten()
        .filter_map(|env| {
            let name = env.get("name")?.as_str()?.trim();
            if name.is_empty() {
                return None;
            }
            let version = env
                .get("dependencies")
                .and_then(|d| d.as_array())
                .and_then(|deps| {
                    deps.iter()
                        .find(|d| d.get("name").and_then(|n| n.as_str()) == Some(name))
                })
                .and_then(|d| d.get("version"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty());
            Some(match version {
                Some(v) => Package::with_version(name, v, backend),
                None => Package::new(name, backend),
            })
        })
        .collect::<Vec<_>>();
    // An empty array is pixi saying it has no global environments, which is a real answer. A
    // non-empty array none of whose entries carried a usable `name` is a schema change, and so
    // is a document that is not the array at all.
    crate::parsers::or_unrecognised_json(
        backend,
        found,
        envs.map(Vec::len),
        "JSON that is not the array of environments this reads, or an array of environments \
         none of them carrying a `name`",
        output,
    )
}

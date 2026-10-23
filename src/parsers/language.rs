use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult, Unrecognised};
use crate::utils::text::sanitize;
use serde_json::Value;

/// The nine language managers' installed listings, dispatched by name.
///
/// **The judgement is made once, here, rather than nine times below.** Each inner reader is a
/// `Vec`-returning parse of one manager's shape, and every one of them has the same way of
/// failing — a `serde_json` call ending in `unwrap_or_default()`, or a tree walk that matches
/// nothing — which spells *"I did not understand this"* as *"nothing is installed"*. Wrapping
/// the dispatch catches all nine at once and needs no change inside any of them.
///
/// It also closes the tenth, which was the widest: `_ => vec![]` answered *the machine is
/// empty* for a backend nobody had written a reader for.
///
/// **The `--json` readers make the judgement themselves and return early**, because a line count
/// is the wrong question to ask of a document: see [`crate::parsers::or_unrecognised_json`]. The
/// rest fall through to the line-counting rule below.
pub fn parse_installed(backend: &str, output: &str) -> ParseResult {
    let clean = sanitize(output);
    // **Who owns the empty answer, decided by backend rather than for all nine.** The JSON
    // readers enforce "empty is an error" through `or_unrecognised_json` — a crashed manager
    // that answered with zero bytes is not a machine with nothing installed — so empty input
    // reaches them and is refused there. The line readers keep the early answer: bun, cargo,
    // gem are legitimately silent on a machine with nothing installed, and yarn's
    // chrome means it never is.
    let json_style = matches!(backend, "npm" | "pnpm" | "pip" | "pipx" | "composer");
    if clean.is_empty() && !json_style {
        return Ok(vec![]);
    }

    let found = match backend {
        "npm" | "pnpm" => return parse_npm_style_json(&clean, backend),
        // `bun pm ls -g` prints an ASCII tree (a header path line + "├── name@ver"
        // rows), NOT npm's `--json` object — routing it through the JSON parser
        // silently returned nothing, which broke list AND made `info`/`remove`
        // no-ops (remove is gated on `info`), leaving stale manifest entries.
        "bun" => {
            // The header carries the truth about zero: `/root/.bun/… node_modules (0)` with
            // no rows under it is an EMPTY machine, and without reading that count the
            // candidate list (the header itself) outnumbers zero rows and `or_unrecognised`
            // refuses — every empty bun machine read as unreadable instead of empty.
            if bun_header_count_is_zero(&clean) {
                return Ok(vec![]);
            }
            return or_unrecognised(
                backend,
                parse_bun_list(&clean),
                &crate::parsers::data_lines(&clean),
            );
        }
        "pip" => return parse_pip_json(&clean),
        "pipx" => return parse_pipx_json(&clean),
        "cargo" => parse_cargo_list(&clean),
        // yarn brackets every command with its own chrome — `yarn global v1.22.22` on the way in
        // and `Done in 0.67s.` on the way out — so an EMPTY global install still prints two
        // lines. Neither is prose by the general rule (no trailing colon, no leading digit, and
        // "Done in 0.67s." is a three-word sentence), so the unread check counted them as rows it
        // had failed to read and `shall list` warned about yarn on every run of a machine with no
        // global packages. Measured on a real Windows box, 2026-08-07.
        //
        // Dropped here rather than in `is_prose_line`, because "what yarn prints around its
        // answer" is knowledge about yarn, and the general rule is for what every manager does.
        "yarn" => {
            let body: String = clean
                .lines()
                .filter(|l| !is_yarn_chrome(l))
                .collect::<Vec<_>>()
                .join("\n");
            return or_unrecognised(
                backend,
                parse_yarn_list(&body),
                &crate::parsers::data_lines(&body),
            );
        }
        "gem" => parse_gem_list(&clean),
        "composer" => return parse_composer_json(&clean),
        other => {
            return Err(Unrecognised {
                backend: other.to_string(),
                data_lines: 0,
                sample: "no installed-listing reader is wired for this backend".into(),
            })
        }
    };

    or_unrecognised(backend, found, &crate::parsers::data_lines(&clean))
}

pub fn parse_search(backend: &str, output: &str) -> Vec<Package> {
    let clean = sanitize(output);
    match backend {
        "cargo" => parse_cargo_search(&clean),
        "gem" => parse_gem_search(&clean),
        "composer" => parse_composer_search(&clean),
        _ => vec![],
    }
}

/// Handles JSON dependencies for NPM and PNPM global lists. npm emits a single object
/// `{"dependencies": {...}}`, but `pnpm ls -g --json` emits an ARRAY of such objects
/// (`[{"dependencies": {...}}]`) — so normalize to a list of entries and pull each one's
/// dependency map, or pnpm's global packages parse as empty.
///
/// **`dependencies` absent is how npm says "nothing".** This rule used to be the opposite —
/// *absent is not empty*, on the reasoning that npm renaming the key would otherwise read as a
/// machine with nothing installed — and the reasoning was sound about a risk that is not the one
/// that happened. Measured in the fedora image, 2026-08-10:
///
/// ```text
/// $ npm list -g --depth=0 --json      # nothing installed globally
/// {
///   "name": "lib"
/// }                                    # exit 0, and no `dependencies` key at all
/// ```
///
/// So the guard fired on the ordinary state of a fresh Node install, `check health` reported
/// **`[FAIL] npm — says it is ready but cannot list`**, and npm dropped out of the READY set on
/// every machine with no global packages. It cost a real backend lifecycle in CI and would cost
/// a first-time user their npm backend entirely.
///
/// **The format-change protection stays, and it is what makes this safe to change.** The two
/// cases are distinguishable by looking, which the first attempt at this fix missed and
/// `a_renamed_container_is_a_format_change_and_not_an_empty_machine` caught:
///
/// | document | reading |
/// |---|---|
/// | `{"name":"lib"}` | zero packages — nothing here could be the map |
/// | `{"name":"root","packages":{"typescript":{…}}}` | **refused** — `packages` is a map of objects, which is what `dependencies` renamed would look like |
///
/// A missing `dependencies` therefore means *empty* only when no other key holds a non-empty
/// object of objects. That is the difference between "this machine has nothing" and "npm
/// changed its schema", and getting it wrong in the permissive direction is the catastrophic
/// one: the planner reads an empty listing as *install everything, own nothing*.
///
/// A document that is not an object or an array is still unrecognised: that is a shape nobody
/// has ever seen from these three, and "empty" is not a reasonable reading of it.
pub fn parse_npm_style_json(output: &str, backend: &str) -> ParseResult {
    let Some(json) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json(backend, vec![], None, "not JSON", output);
    };
    let mut res = vec![];
    let entries: Vec<&Value> = match &json {
        Value::Array(items) => items.iter().collect(),
        // pnpm emits an array of these, npm a single object. Anything else is a shape this
        // parser has no reading for, and it says so rather than calling it empty.
        Value::Object(_) => vec![&json],
        _ => {
            return crate::parsers::or_unrecognised_json(
                backend,
                vec![],
                None,
                "JSON that is neither an object nor an array",
                output,
            )
        }
    };
    let mut declared = 0;
    let mut renamed = false;
    for entry in entries {
        match entry.get("dependencies").and_then(|d| d.as_object()) {
            Some(deps) => {
                declared += deps.len();
                for (name, val) in deps {
                    let version = val
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    res.push(Package::with_version(name, version, backend));
                }
            }
            // No `dependencies`. That is npm's empty answer — *unless* the document carries
            // some other key holding what a package map looks like, which is what a renamed
            // container looks like and is the one thing that must never read as "empty".
            None => {
                renamed |= entry.as_object().is_some_and(|m| {
                    m.values().any(|v| {
                        v.as_object().is_some_and(|inner| {
                            !inner.is_empty() && inner.values().all(Value::is_object)
                        })
                    })
                })
            }
        }
    }
    crate::parsers::or_unrecognised_json(
        backend,
        res,
        // `Some(0)` — an answer of none — only when nothing in the document could have been the
        // map under another name. `None` keeps the refusal.
        if renamed { None } else { Some(declared) },
        "JSON with no `dependencies` object",
        output,
    )
}

/// Parses the flat JSON array output of `pip list --format=json`.
fn parse_pip_json(output: &str) -> ParseResult {
    let Some(json) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json("pip", vec![], None, "not JSON", output);
    };
    let entries = json.as_array();
    let found = entries
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let name = p.get("name")?.as_str()?;
                    let ver = p.get("version")?.as_str()?;
                    Some(Package::with_version(name, ver, "pip"))
                })
                .collect()
        })
        .unwrap_or_default();
    crate::parsers::or_unrecognised_json(
        "pip",
        found,
        entries.map(Vec::len),
        "JSON, but not the array `pip list --format=json` returns",
        output,
    )
}

/// Parses the complex JSON object of `pipx list --json`.
fn parse_pipx_json(output: &str) -> ParseResult {
    let Some(json) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json("pipx", vec![], None, "not JSON", output);
    };
    let venvs = json.get("venvs").and_then(|v| v.as_object());
    let mut res = vec![];
    if let Some(venvs) = venvs {
        for (name, data) in venvs {
            // `package_version`, which is what pipx calls it. Reading `version` — a key that
            // schema does not have — gave every pipx package the version `unknown`, on every
            // machine, silently: `list` printed it, and `lock` skips a version reading
            // `unknown`, so `shall lock` pinned no pipx package and said nothing. Caught by the
            // first fixture captured from the tool itself.
            let ver = data
                .get("metadata")
                .and_then(|m| m.get("main_package"))
                .and_then(|p| p.get("package_version"))
                .and_then(|v| v.as_str());
            res.push(Package::with_version(
                name,
                ver.unwrap_or("unknown"),
                "pipx",
            ));
        }
    }
    crate::parsers::or_unrecognised_json(
        "pipx",
        res,
        venvs.map(serde_json::Map::len),
        "JSON with no `venvs` object",
        output,
    )
}

/// Parses the formatted text list of `cargo install --list`.
/// Format: "name v1.2.3:"
pub fn parse_cargo_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter(|l| l.contains(" v") && l.ends_with(':'))
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(Package::with_version(
                    parts[0],
                    parts[1].trim_matches(|c| c == 'v' || c == ':'),
                    "cargo",
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Parses the ASCII tree output of `yarn global list`.
///
/// Top-level rows only, for the reason [`crate::parsers::ecosystem::pixi_list`] gives: in every
/// one of these trees an indented row is a child of the row above — a dependency, or a property
/// of the entry — and a parser that trims the indentation away before reading reports it as
/// something the user installed.
/// `yarn global list` — yarn 1, the only yarn that has `global` at all.
///
/// The package and its version appear in exactly one place, on the line yarn labels `info`:
///
/// ```text
/// yarn global v1.22.22
/// info "catj@1.0.4" has binaries:
///    - catj
/// Done in 0.07s.
/// ```
///
/// This parser used to be given `--json` and written for an ASCII tree, and yarn emits
/// neither. Measured on a host with `catj` installed: the JSON stream's only `list` record is
/// `{"type":"list","data":{"type":"bins-catj","items":["catj"]}}` — the *binaries*, not the
/// package — and the plain output has no tree either. So the filter that dropped every line
/// containing `info` dropped the one line that carries the answer, and `shall list -b yarn`
/// returned nothing on a machine with yarn packages on it.
///
/// That is not only `list`: `remove` is gated on `info`, which reads this same listing, so a
/// declared yarn package could not be removed and its manifest line went stale. **bun had the
/// identical bug for the identical reason** (see `parse_bun_list`) — a parser written for a
/// format the tool does not print, and nothing between them noticing, because an empty listing
/// looks exactly like an empty machine.
/// yarn's own banner and footer, which every `yarn` command prints whatever the answer is.
fn is_yarn_chrome(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("yarn global v") || t.starts_with("Done in ") || t.starts_with("warning ")
}

pub fn parse_yarn_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter_map(|l| {
            // `info "<name>@<version>" has binaries:` / `... has no binaries`. The quotes are
            // what separate a package line from yarn's other `info` chatter, which carries none.
            let (spec, _) = l
                .trim()
                .strip_prefix("info ")?
                .strip_prefix('"')?
                .split_once('"')?;
            let (name, ver) = spec.rsplit_once('@')?;
            (!name.is_empty() && !ver.is_empty()).then(|| Package::with_version(name, ver, "yarn"))
        })
        .collect()
}

/// Parses the ASCII-tree output of `bun pm ls -g`.
/// Format: a header path line (e.g. "/root/.bun/install/global node_modules (2)")
/// followed by "├── name@version" / "└── name@version" rows. Scoped packages keep
/// their leading '@' in the name (e.g. "@scope/pkg@1.2.3" -> name "@scope/pkg").
/// The header has no '@', so it is filtered out.
///
/// A row indented past the first column is a dependency of the one above it — `bun pm ls -g
/// --all` prints four levels — and it is not globally installed. One flag away from forty
/// invented packages, which is `pixi global list`'s `exposes:` row in another tool.
/// The `(N)` at the end of bun's header path line, when the line is there at all.
///
/// `/root/.bun/install/global node_modules (2)` — npm-style count of what the listing holds.
fn bun_header_count_is_zero(output: &str) -> bool {
    output
        .lines()
        .filter_map(|l| l.trim_end().rsplit_once('('))
        .any(|(before, tail)| {
            before.trim_end().ends_with("node_modules")
                && match tail.strip_suffix(')') {
                    Some(n) => n.trim().parse::<u64>().ok() == Some(0),
                    None => false,
                }
        })
}

pub fn parse_bun_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter(|l| l.contains('@'))
        .filter(|l| !l.starts_with(char::is_whitespace) && !l.starts_with('│'))
        .filter_map(|l| {
            let cleaned = l
                .trim()
                .trim_start_matches(|c: char| {
                    c.is_whitespace() || matches!(c, '├' | '└' | '│' | '─')
                })
                .trim();
            let (name, ver) = cleaned.rsplit_once('@')?;
            if name.is_empty() || ver.is_empty() || ver.contains(char::is_whitespace) {
                return None;
            }
            Some(Package::with_version(name, ver, "bun"))
        })
        .collect()
}

/// Parses the text output of `gem list --local`.
/// Format: "name (1.2.3, 1.1.0)"
fn parse_gem_list(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("***"))
        .filter_map(|line| {
            let (name, rest) = line.split_once(' ')?;
            // The bracket rule is `parsers::utils`'s, not a second copy of it. The fallback is
            // the old behaviour for a line without them: a wrong version is drift, a dropped
            // package is a removal.
            let inside = crate::parsers::utils::extract_version_bracketed(rest.trim())
                .unwrap_or_else(|| rest.trim().to_string());
            let ver = inside.split(',').next()?;
            // `bundler (default: 4.0.10)` — RubyGems marks the gems that ship with Ruby,
            // and the marker is not part of the version. Kept as one, `shall list` printed
            // `default: 4.0.10` in its version column: an `@version=` can never match it,
            // and `list --outdated` shows it beside a real version, which reads as Shall
            // not knowing what is installed.
            let ver = ver.trim().strip_prefix("default:").unwrap_or(ver).trim();
            Some(Package::with_version(name.trim(), ver, "gem"))
        })
        .collect()
}

/// Parses the JSON output of `composer global show --format=json`.
///
/// Composer answers two different shapes for the same question. With packages installed it
/// emits `{"installed": [{name, version, ...}, ...]}`; with none it emits a bare `[]`. The
/// populated shape is the one the reader below is named for; the empty one must read as *zero
/// packages* rather than as a format change — a machine with nothing installed is the common
/// case, and refusing it would drop composer out of the READY set and out of every real
/// lifecycle, which is exactly what happened when the tools image seeded an empty global
/// project and `check health` reported composer CRITICAL.
pub fn parse_composer_json(output: &str) -> ParseResult {
    let Some(json) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json("composer", vec![], None, "not JSON", output);
    };
    // `composer global show --format=json` answers `[]` on an empty global project. A top-level
    // array IS the empty installed list — composer's own empty answer, not a renamed schema.
    if let Value::Array(items) = &json {
        if items.is_empty() {
            return crate::parsers::or_unrecognised_json(
                "composer",
                vec![],
                Some(0),
                "JSON with no `installed` array",
                output,
            );
        }
        // A non-empty top-level array is a shape composer has never produced; say so rather
        // than guess a reading for it.
        return crate::parsers::or_unrecognised_json(
            "composer",
            vec![],
            Some(items.len()),
            "composer answered a bare array that is not empty",
            output,
        );
    }
    let installed = json.get("installed").and_then(|i| i.as_array());
    let mut res = vec![];
    for pkg in installed.into_iter().flatten() {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        // A missing version is version-less, never `Some("")` — which poisons plan
        // comparison (apt.rs documents the shape).
        match pkg.get("version").and_then(|v| v.as_str()) {
            Some(ver) => res.push(Package::with_version(name, ver, "composer")),
            None => res.push(Package::new(name, "composer")),
        }
    }
    crate::parsers::or_unrecognised_json(
        "composer",
        res,
        installed.map(Vec::len),
        "JSON with no `installed` array",
        output,
    )
}

/// Specialized parser for `cargo search`.
fn parse_cargo_search(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let (name, _) = l.split_once('=')?;
            Some(Package::new(name.trim(), "cargo"))
        })
        .collect()
}

/// Specialized parser for `gem search`.
fn parse_gem_search(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("***"))
        .filter_map(|l| {
            let (name, _) = l.split_once(' ')?;
            Some(Package::new(name.trim(), "gem"))
        })
        .collect()
}

/// Specialized parser for `composer search`.
fn parse_composer_search(output: &str) -> Vec<Package> {
    let json = crate::parsers::json_document(output).unwrap_or_default();
    json.as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|p| {
            let name = p.get("name")?.as_str()?;
            Some(Package::new(name, "composer"))
        })
        .collect()
}

/// `pip list --outdated --format=json` (`Q44`).
///
/// ```json
/// [{"name": "pip", "version": "26.1.2", "latest_version": "26.2.1", "latest_filetype": "wheel"}]
/// ```
///
/// `latest_version`, never `version` — the second is what is installed, and reporting it as the
/// available one makes every package look current.
pub fn parse_pip_outdated(output: &str) -> Vec<Package> {
    let Ok(items) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let Some(items) = items.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|p| {
            let name = p.get("name")?.as_str()?.trim();
            let latest = p.get("latest_version")?.as_str()?.trim();
            if name.is_empty() || latest.is_empty() {
                return None;
            }
            Some(Package::with_version(name, latest, "pip"))
        })
        .collect()
}

/// `npm outdated -g --json` / `pnpm outdated -g --json` (`Q44`).
///
/// ```json
/// {"typescript": {"current": "5.4.0", "wanted": "5.6.0", "latest": "5.7.2"}}
/// ```
///
/// **`latest`, not `wanted`.** `wanted` is the newest release satisfying the range in a
/// package.json, and a global install has no package.json to constrain it — reporting `wanted`
/// would hide a major version behind a caret nobody wrote.
///
/// `npm outdated` exits non-zero when it *finds* something, which is why this is read through
/// `run_output` rather than a status-checked reader.
pub fn parse_npm_outdated(output: &str, backend: &str) -> Vec<Package> {
    let Ok(doc) = serde_json::from_str::<Value>(output) else {
        return Vec::new();
    };
    let Some(map) = doc.as_object() else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(name, v)| {
            let latest = v.get("latest")?.as_str()?.trim();
            if name.trim().is_empty() || latest.is_empty() {
                return None;
            }
            Some(Package::with_version(name.trim(), latest, backend))
        })
        .collect()
}

/// `gem outdated`: `name (installed < latest)`, one per line (`Q44`).
pub fn parse_gem_outdated(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (name, rest) = line.split_once(" (")?;
            let latest = rest.trim_end_matches(')').split('<').nth(1)?.trim();
            if name.is_empty() || latest.is_empty() {
                return None;
            }
            Some(Package::with_version(name.trim(), latest, "gem"))
        })
        .collect()
}

/// `composer global outdated --format=json` (`Q44`).
///
/// ```json
/// {"installed":[{"name":"psr/log","version":"1.1.4","latest":"3.0.2",
///                "latest-status":"update-possible"}]}
/// ```
///
/// **`latest`, not `version`** — the second is what is installed. And the JSON is *found*, not
/// assumed: composer prints `Changed current directory to /root/.composer` ahead of it, so a
/// strict parse fails and reports nothing outdated on every machine with a global config dir,
/// which is all of them.
pub fn parse_composer_outdated(output: &str) -> Vec<Package> {
    let text = sanitize(output);
    let Some(doc) = crate::parsers::json_document(&text) else {
        return Vec::new();
    };
    let Some(items) = doc.get("installed").and_then(|i| i.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|p| {
            let name = p.get("name")?.as_str()?.trim();
            let latest = p.get("latest")?.as_str()?.trim();
            if name.is_empty() || latest.is_empty() {
                return None;
            }
            Some(Package::with_version(name, latest, "composer"))
        })
        .collect()
}

//! **A `pub fn` nothing calls is invisible to every gate this repository owns.**
//!
//! `cargo` never warns about an unused `pub fn` in a library — it is public, so something
//! outside the crate might call it. Nothing outside this crate does: `shall` is an application
//! whose library half exists so the tests can reach it. So `dead_code` is off for exactly the
//! class of dead code that costs something here, and the build, clippy, fmt and the whole test
//! suite all pass over it.
//!
//! **What it cost.** `MetricsCollector::record_error` was the only writer of `Metrics.errors`
//! and had zero callers anywhere in the tree. `errors.is_empty()` was therefore permanently
//! true, which made the transaction summary's `Status:` line the constant `SUCCESS`, made
//! `DEGRADED` unreachable, and made `print_summary_quiet` — whose entire body printed the
//! unreachable error block — print nothing under any circumstances. A sync in which every
//! package failed reported success, and the same run under `--quiet` printed zero bytes. One
//! uncalled function, three broken outputs, and 517 passing tests (B1).
//!
//! Six more were found by the same scan and deleted with it: `App::new_with_executor`,
//! `Sandbox::is_supported`, `ShimManager::get_bin_dir`, `FormatOrder::rejected_by`,
//! `File::module_blocks`, `Phase::is_work`.
//!
//! **The bar here is zero references, not zero production callers.** A `pub fn` reached only
//! from tests is usually a test double — the fake executor's API alone accounts for a hundred
//! references — and policing those would make this a gate people delete. A function nothing
//! mentions at all is unambiguous.
//!
//! **The first version of this gate could not fail for a whole class, and it collected.** It
//! recorded declarations in a `BTreeMap<name, site>`, so two functions sharing a name collapsed
//! into one row and their references were counted together — the file called that "the safe
//! direction", and it is the opposite. One live `discover` vouched for every dead `discover`.
//! `Layout::discover` sat in the tree with no caller at all, re-implementing root resolution
//! with one ordering rule missing and no empty-value filter (`#219`), and this gate passed over
//! it for as long as it existed. It was found by reading, not by the gate.
//!
//! **So a name is no longer the unit of the claim; a declaration is.** For a name declared more
//! than once, evidence has to be attributable to *that* declaration: a call that names its type
//! (`Layout::discover`), a `Self::discover` in its own file, a use inside its own file, or a
//! `.discover(` in a file that declares no function of that name. The last is the weakest and
//! it is here on purpose — almost every method in this tree is reached by a receiver call, and a
//! scan that ignored those would report forty live methods as dead. What it cannot do is vouch
//! for a declaration from a file that declares the same name itself, which is exactly how
//! `Layout::discover` hid: every `.discover()` in the tree sat in `adopt.rs`, beside
//! `Adoptor::discover`. Whatever is still unattributable is reported and pinned, so the set can
//! fall and never rise.
//!
//! `the_scan_can_see_a_dead_twin` feeds the scan a tree built to contain that exact case, because
//! the gate that missed it had no such test.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Every `.rs` file under a directory.
fn rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn repo(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// The identifier a `pub fn` / `pub async fn` declaration names, if the line is one.
///
/// Textual, and that is sound for the question being asked: a trait method cannot carry `pub`
/// in its impl, so every hit is an inherent or free function — the two kinds that can go
/// uncalled without the compiler minding.
fn declared_name(line: &str) -> Option<&str> {
    let t = line.trim_start();
    let rest = t
        .strip_prefix("pub fn ")
        .or_else(|| t.strip_prefix("pub async fn "))
        .or_else(|| t.strip_prefix("pub(crate) fn "))
        .or_else(|| t.strip_prefix("pub(crate) async fn "))?;
    let name = rest
        .split(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .next()?;
    (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')).then_some(name)
}

/// The type an `impl` block belongs to, from its header line.
///
/// `impl Layout {`, `impl<'a> Foo<'a> {` and `impl Foo for Bar {` all reduce to the name a
/// caller spells: `Bar::new`. Lifetimes and type parameters are removed first, because
/// `impl<'a> Resolver<'a>` would otherwise name its owner `a`.
fn impl_owner(line: &str) -> Option<String> {
    let t = line.trim_start();
    let head = t.strip_prefix("impl")?.split('{').next()?;
    if head.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let mut plain = String::with_capacity(head.len());
    let mut depth = 0usize;
    for c in head.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => plain.push(c),
            _ => {}
        }
    }
    plain
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .rfind(|s| !s.is_empty())
        .map(str::to_string)
}

/// How every identifier in the tree is spelled, counted once.
///
/// The scan asks four questions per declaration — is its name mentioned at all, is it spelled
/// `Type::name`, is `Self::name` used beside it, is a `.name(` call outside the files that
/// declare that name — and asking them of the raw text each time is quadratic: 3,400
/// declarations against 5 MB is minutes, not seconds. One tokenizing pass answers all four.
#[derive(Default)]
struct Spellings {
    bare: BTreeMap<String, usize>,
    dotted: BTreeMap<String, usize>,
    qualified: BTreeMap<String, usize>,
    per_file: BTreeMap<(PathBuf, String), (usize, usize)>,
}

fn spellings(sources: &[(PathBuf, String)]) -> Spellings {
    let mut out = Spellings::default();
    for (path, body) in sources {
        let bytes = body.as_bytes();
        let mut i = 0usize;
        let mut prev: Option<String> = None;
        let mut gap_start = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_alphanumeric() || b == b'_' {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let token = &body[start..i];
                let gap = &body[gap_start..start];
                *out.bare.entry(token.to_string()).or_default() += 1;
                let slot = out
                    .per_file
                    .entry((path.clone(), token.to_string()))
                    .or_default();
                slot.0 += 1;
                if gap.ends_with('.') {
                    *out.dotted.entry(token.to_string()).or_default() += 1;
                    slot.1 += 1;
                } else if gap.ends_with("::") {
                    if let Some(owner) = &prev {
                        *out.qualified
                            .entry(format!("{owner}::{token}"))
                            .or_default() += 1;
                    }
                }
                prev = Some(token.to_string());
                gap_start = i;
            } else {
                i += 1;
            }
        }
    }
    out
}

struct Declaration {
    name: String,
    owner: String,
    file: PathBuf,
    at: String,
}

/// The declarations a set of `(path, source)` pairs holds, and the two lists they produce.
///
/// Pure in its input, so the analysis can be handed a tree built to contain a defect.
fn orphans(sources: &[(PathBuf, String)], root: &Path) -> (Vec<String>, Vec<String>) {
    let spell = spellings(sources);
    let mut declarations: Vec<Declaration> = Vec::new();
    let mut per_name: BTreeMap<String, usize> = BTreeMap::new();
    let mut declarers: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for (path, body) in sources {
        if !path.starts_with(root) {
            continue;
        }
        let mut owner = String::new();
        for (i, line) in body.lines().enumerate() {
            // An impl's closing brace is at column 0 under `cargo fmt`, and everything after it is
            // free again. Without this a free function inherits the type above it and is judged
            // by the method rule — `for_manager` in `model/prereq.rs` was reported as
            // `PrereqDef::for_manager` for exactly that reason.
            if line.trim_end() == "}" {
                owner.clear();
                continue;
            }
            if let Some(found) = impl_owner(line) {
                owner = found;
                continue;
            }
            if let Some(name) = declared_name(line) {
                per_name
                    .entry(name.to_string())
                    .and_modify(|n| *n += 1)
                    .or_insert(1);
                declarers
                    .entry(name.to_string())
                    .or_default()
                    .push(path.clone());
                declarations.push(Declaration {
                    name: name.to_string(),
                    owner: if owner.is_empty() {
                        String::from("(free)")
                    } else {
                        owner.clone()
                    },
                    file: path.clone(),
                    at: format!(
                        "{}:{}",
                        path.strip_prefix(root).unwrap_or(path).display(),
                        i + 1
                    ),
                });
            }
        }
    }

    let per_file = |file: &Path, name: &str| -> (usize, usize) {
        spell
            .per_file
            .get(&(file.to_path_buf(), name.to_string()))
            .copied()
            .unwrap_or((0, 0))
    };

    let mut out: Vec<String> = Vec::new();
    let mut twins: Vec<String> = Vec::new();
    for decl in &declarations {
        // `main` is called by the runtime.
        if decl.name == "main" {
            continue;
        }
        let label = format!("{}::{}  ({})", decl.owner, decl.name, decl.at);
        let shared = per_name.get(&decl.name).copied().unwrap_or(1) > 1;
        let (own, _) = per_file(&decl.file, &decl.name);
        let bare = spell.bare.get(&decl.name).copied().unwrap_or(0);

        // A free function is called by its bare name, so a mention anywhere is the only evidence
        // there can be, and one mention is the declaration itself.
        if !shared || decl.owner == "(free)" {
            if bare <= 1 {
                out.push(label);
            }
            continue;
        }

        let qualified = spell
            .qualified
            .get(&format!("{}::{}", decl.owner, decl.name))
            .copied()
            .unwrap_or(0)
            + spell
                .qualified
                .get(&format!("Self::{}", decl.name))
                .copied()
                .unwrap_or(0);
        let dotted = spell.dotted.get(&decl.name).copied().unwrap_or(0);
        let mine: usize = declarers
            .get(&decl.name)
            .map(|files| files.iter().map(|f| per_file(f, &decl.name).1).sum())
            .unwrap_or(0);
        let receiver_elsewhere = dotted.saturating_sub(mine);

        if qualified == 0 && own <= 1 && receiver_elsewhere == 0 {
            // The line number is left out of the pinned name on purpose: a ratchet that has to
            // be edited every time a line moves is a ratchet people edit without reading.
            twins.push(format!("{}::{}  ({})", decl.owner, decl.name, decl.at));
        }
    }
    (out, twins)
}

/// The scan sees a dead twin: two functions named `discover`, one called, one not.
///
/// This is the case the first version of this gate passed over in the real tree, in exactly
/// these words, and it is here so the gate cannot go back to passing over it.
#[test]
fn the_scan_can_see_a_dead_twin() {
    let root = PathBuf::from("src");
    let a = root.join("a.rs");
    let b = root.join("b.rs");
    let c = root.join("c.rs");

    // `A::discover` is reached by a path that names its type; `B::discover` by nothing at all.
    // Under the first version of this gate the *name* `discover` was mentioned, so both were
    // called and neither was reported.
    let sources = vec![
        (
            a.clone(),
            "impl A {\n    pub fn discover() -> usize { 1 }\n}\n".to_string(),
        ),
        (
            b.clone(),
            "impl B {\n    pub fn discover() -> usize { 2 }\n}\n".to_string(),
        ),
        (
            c.clone(),
            "fn use_a() -> usize { A::discover() }\n".to_string(),
        ),
    ];
    let (found, twins) = orphans(&sources, &root);
    assert!(found.is_empty(), "neither is definitely dead: {found:?}");
    assert_eq!(twins.len(), 1, "one twin is unattributed: {twins:?}");
    assert!(
        twins[0].starts_with("B::discover") && twins[0].contains("b.rs"),
        "the unattributed one is `B::discover`, named with its type and where it lives, because \
         that is what a reader needs to go and look: {twins:?}"
    );

    // A receiver call in its own file is a use. One in a file that declares nothing by this name
    // is weaker evidence — it says *a* `discover` is called — and text cannot attribute it to a
    // type, which is why it is the one clause that still lets a twin through.
    let in_own_file = vec![
        (
            a.clone(),
            "impl A {\n    pub fn discover() -> usize { 1 }\n    fn call(&self) -> usize { self.discover() }\n}\n"
                .to_string(),
        ),
        (
            b.clone(),
            "impl B {\n    pub fn discover() -> usize { 2 }\n}\n".to_string(),
        ),
    ];
    let (found, twins) = orphans(&in_own_file, &root);
    assert!(found.is_empty(), "{found:?}");
    assert_eq!(
        twins.len(),
        1,
        "only b.rs is unattributed — `self.discover()` in a.rs is A's own use: {twins:?}"
    );

    let receiver_elsewhere = vec![
        (
            a.clone(),
            "impl A {\n    pub fn discover() -> usize { 1 }\n}\n".to_string(),
        ),
        (
            b.clone(),
            "impl B {\n    pub fn discover() -> usize { 2 }\n}\n".to_string(),
        ),
        (
            c.clone(),
            "fn use_one(b: &B) -> usize { b.discover() }\n".to_string(),
        ),
    ];
    let (_, twins) = orphans(&receiver_elsewhere, &root);
    assert!(
        twins.is_empty(),
        "a receiver call outside both declaring files keeps both alive — the residual \
         blindness, stated rather than discovered later: {twins:?}"
    );

    // A unique name with no mention anywhere is a verdict, not a watch.
    let unique = vec![(
        c.clone(),
        "impl C {\n    pub fn nothing_calls_this() -> usize { 3 }\n}\n".to_string(),
    )];
    let (found, twins) = orphans(&unique, &root);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("C::nothing_calls_this"), "{found:?}");
    assert!(twins.is_empty(), "{twins:?}");
}

/// The names this gate cannot attribute, pinned so the set can fall and never rise.
///
/// A text-only scan cannot tell `lock.retain_declared(..)` from a call on any other type, and a
/// serde `#[serde(with = "resource_kind")]` names its serializer as a *string*, so no code
/// mentions it. Those are honest limits, and a limit with no number is the thing this repository
/// keeps finding behind a green run.
///
/// **A row leaves when the function gains a qualified caller or is deleted. A new row means a
/// new `pub fn` that nothing attributes — read it before adding it here**, because that is the
/// shape of `Layout::discover`, which this gate passed over for its whole life (`#219`).
const UNATTRIBUTED: &[&str] = &[];

fn assert_twins_ratchet(twins: &[String]) {
    let mut known: Vec<String> = UNATTRIBUTED.iter().map(|s| s.to_string()).collect();
    known.sort();
    known.dedup();
    let mut got: Vec<String> = twins
        .iter()
        .map(|t| t.split("  (").next().unwrap_or(t).to_string())
        .collect();
    got.sort();
    got.dedup();
    assert_eq!(
        got, known,
        "the set of public functions this scan cannot attribute has changed. Paste the list \
         above into `UNATTRIBUTED` only after reading each one: a name that appears because a new \
         `pub fn` was added beside an existing one is the shape of `Layout::discover`, which this \
         gate passed over for its whole life (`#219`).\n  now: {got:#?}\n  known: {known:#?}"
    );
}

#[test]
fn no_public_function_is_referenced_by_nothing_at_all() {
    let mut files = Vec::new();
    rust_files(&repo("src"), &mut files);
    let src_count = files.len();
    rust_files(&repo("tests"), &mut files);
    assert!(
        src_count > 50 && files.len() > src_count,
        "the scan found {src_count} source file(s) and {} in total — it is not reading the \
         tree, so every assertion below is vacuous",
        files.len()
    );

    let sources: Vec<(PathBuf, String)> = files
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|b| (p, b)))
        .collect();

    let declared = sources
        .iter()
        .filter(|(p, _)| p.starts_with(repo("src")))
        .flat_map(|(_, b)| b.lines().filter_map(declared_name))
        .count();
    assert!(
        declared > 300,
        "only {declared} public function(s) found; the declaration scan has stopped matching the \
         tree"
    );

    let (found, twins) = orphans(&sources, &repo("src"));
    assert!(
        found.is_empty(),
        "these public functions are referenced by nothing in the tree — not by production \
         code, not by a test, not by a doc comment. `cargo` cannot see this class, which is \
         how a `record_error` with no callers made `DEGRADED` unreachable and silenced \
         `--quiet` entirely (B1), and how `Layout::discover` survived with no caller at all \
         (`#219`). Call each one, or delete it:\n  {}",
        found.join("\n  ")
    );
    assert_twins_ratchet(&twins);
}

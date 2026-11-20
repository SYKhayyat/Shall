//! **The allowance half of #204: an `#[allow(clippy::too_many_lines)]` outlives its issue unless something notices.**
//!
//! The build already enforces the threshold: `cargo clippy -D warnings` treats `too_many_lines`
//! as an error, so a function over 250 lines cannot compile without an allow. What the build
//! cannot see is the allow itself: it turns the build green without shortening anything, and
//! nothing in the tree notices that the allow has outlived the issue that will remove it.
//!
//! **This gate audits the allowance table, not the line count.** The count is clippy's, and
//! reimplementing it here would be a second opinion about the same number. What is asserted is
//! that the tree and the table agree in both directions: every allow is a row, and every row is
//! an allow. A row that has been earned back — the split landed and the allow came off — is
//! deleted in the same commit, and the gate reds if it was not.
//!
//! **There are no exemptions left, and the table stays because the rule is about the next one.**
//! The four allows it carried — `Transaction::execute_batch_with_retry` (#199) and `heal`,
//! `GithubInstallable::install` and `dispatch` (#205) — were all earned back by splitting the
//! functions. A new allow is red until somebody writes a row naming the issue that removes it.
//!
//! **Four lints are forbidden outright**, not tabled: `too_many_arguments`,
//! `fn_params_excessive_bools`, `type_complexity` and `module_inception`. Each asks for a name —
//! a struct, a type alias, or one rename — so a site has somewhere to go that is not an allow.
//! The last two were earned back by #208: the two `VARS_MEMO`/`why`-snapshot types became structs
//! and `config::config` became `config::declaration`.

use std::path::Path;

/// Every `#[allow(clippy::too_many_lines)]` in the tree, and the issue that will remove it.
const ALLOWED_FOR_NOW: &[(&str, &str, &str)] = &[];

/// Lints no site may silence, and the fix each one is asking for — which is a name, never an
/// allow. The second field is the message a new site gets.
const FORBIDDEN: &[(&str, &str)] = &[
    (
        "#[allow(clippy::too_many_arguments)]",
        "the arguments that travel together become one value with named fields, as \
         `app/machinery.rs` does",
    ),
    (
        "#[allow(clippy::fn_params_excessive_bools)]",
        "the booleans are one enum, as `OptIns` is — not a flag per argument",
    ),
    (
        "#[allow(clippy::type_complexity)]",
        "name the type: a struct with field names for a tuple, or a `type` alias for the part \
         that repeats",
    ),
    (
        "#[allow(clippy::module_inception)]",
        "rename one of the two modules — the lint is the file tree saying the same word twice",
    ),
];

/// The name of the function an allow licenses: the next `fn` after the allow.
fn licensed_function(lines: &[String], at: usize) -> Option<String> {
    for line in lines.iter().skip(at + 1) {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
        if let Some(rest) = t.strip_prefix("pub fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
        if let Some(rest) = t.strip_prefix("async fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
        if let Some(rest) = t.strip_prefix("pub async fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
        if let Some(rest) = t.strip_prefix("pub(crate) fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
        if let Some(rest) = t.strip_prefix("pub(crate) async fn ") {
            return Some(rest.split('(').next()?.trim().to_string());
        }
    }
    None
}

fn rust_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }
    walk(root, &mut out);
    out
}

#[test]
fn every_allow_for_a_long_function_names_the_issue_that_removes_it() {
    let mut found: Vec<(String, String)> = Vec::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for p in rust_files(&root.join("src"))
        .into_iter()
        .chain(rust_files(&root.join("tests")))
    {
        let rel = p
            .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("under the repo")
            .to_string_lossy()
            .to_string();
        let text = std::fs::read_to_string(&p).expect("source is readable");
        let lines: Vec<String> = text.lines().map(str::to_string).collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "#[allow(clippy::too_many_lines)]" {
                let name = licensed_function(&lines, i).unwrap_or_else(|| {
                    panic!("{rel}:{i} carries the allow but no function follows it")
                });
                found.push((rel.clone(), name));
            }
        }
    }

    for (file, fn_name) in &found {
        let row = ALLOWED_FOR_NOW
            .iter()
            .find(|(f, n, _)| f == file && n == fn_name);
        assert!(
            row.is_some(),
            "`{file}` allows `{fn_name}` to be long, and the allowance table has no row for it. \
             A new allow is a permission slip: name the issue that will remove it, or split the \
             function instead."
        );
    }

    // The other direction, which is what stops the table becoming a graveyard: an allow that
    // has been taken off because the split landed must leave.
    for (file, fn_name, issue) in ALLOWED_FOR_NOW {
        assert!(
            found.iter().any(|(f, n)| f == file && n == fn_name),
            "`{file}` lists `{fn_name}` as licensed by `{issue}`, but the allow is gone. The \
             split landed; delete the row in the same commit."
        );
    }
}

#[test]
fn no_site_silences_a_lint_whose_fix_is_a_name() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders: Vec<String> = Vec::new();
    for p in rust_files(&root.join("src")) {
        let rel = p
            .strip_prefix(root)
            .expect("under the repo")
            .to_string_lossy()
            .to_string();
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            let t = line.trim();
            if let Some((allow, fix)) = FORBIDDEN.iter().find(|(allow, _)| *allow == t) {
                offenders.push(format!("{rel}:{}: {}\n             → {fix}", i + 1, allow));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a site silences a lint whose fix is a name:\n  {}",
        offenders.join("\n  ")
    );
}

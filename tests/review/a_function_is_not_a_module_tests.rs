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
//! **The one exemption is a row naming an issue.** `Transaction::execute_batch_with_retry` is
//! #199's job, so its allow names `#199`; the other three name `#205`.

use std::path::Path;

/// Every `#[allow(clippy::too_many_lines)]` in the tree, and the issue that will remove it.
const ALLOWED_FOR_NOW: &[(&str, &str, &str)] = &[
    (
        "src/core/transaction.rs",
        "execute_batch_with_retry",
        "#199",
    ),
    ("src/app/sync/mod.rs", "heal", "#205"),
    ("src/backends/github.rs", "install", "#205"),
    ("src/main.rs", "dispatch", "#205"),
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

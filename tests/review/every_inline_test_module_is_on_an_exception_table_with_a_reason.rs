use crate::ledger::Ledger;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const INLINE_TEST_EXCEPTIONS: &[(&str, &str)] = &[];

fn rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn relative(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn carries_inline_test(body: &str) -> bool {
    body.lines().any(|line| {
        let line = line.trim();
        let Some(attribute) = line.strip_prefix("#[") else {
            return false;
        };
        let attribute = attribute.split(']').next().unwrap_or(attribute);
        (attribute.starts_with("cfg(") || attribute.starts_with("cfg_attr("))
            && attribute.contains("test")
    })
}

#[test]
fn every_inline_test_module_is_on_an_exception_table_with_a_reason() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut found = BTreeSet::new();
    for path in &files {
        let body = std::fs::read_to_string(path).expect("the source file is readable");
        if carries_inline_test(&body) {
            found.insert(relative(path));
        }
    }

    Ledger::of(
        "a source file carrying an inline test module",
        "INLINE_TEST_EXCEPTIONS",
    )
    .pairs(INLINE_TEST_EXCEPTIONS)
    .scanning_at_least(100)
    .audit_allowing_no_findings(files.len(), &found);
}

//! Nothing in `src/` reads argv as `String`.
//!
//! **Why a gate and not the four call sites.** `std::env::args()` unwraps the conversion, so it
//! is not a lossy reader — it is an abort waiting for the first argument that is not UTF-8, and
//! under `panic = "abort"` (the release profile) it is a core dump rather than a message. The
//! four sites that did it were the log level, the two pre-clap flag peeks, and the shim's
//! forward — and the shim's is the one that made it a product defect instead of a crash: a
//! shim is *supposed* to hand its arguments to another program, so a `String` there refuses
//! arguments the target accepts.
//!
//! Fixing four call sites does not stop the fifth. `args_os()` is the only reader this process
//! may use, every consumer takes `OsString`, and a conversion to text has to be explicit
//! (`to_str()`, which can fail, or `to_string_lossy()`, which says what it does). This is what
//! makes that a property of the tree rather than a convention of the four functions.

use std::path::{Path, PathBuf};

const SELF: &str = "no_source_reads_argv_as_utf8_tests.rs";

fn rust_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out
}

/// **The argv reader is `args_os`, everywhere, without exception.**
#[test]
fn no_source_file_unwraps_argv_into_strings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders: Vec<String> = Vec::new();

    for file in rust_files() {
        if file.file_name().is_some_and(|n| n == SELF) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("env::args()") || code.contains("env::args_os().map") {
                offenders.push(format!(
                    "{}:{}: {}",
                    file.strip_prefix(root)
                        .unwrap_or(&file)
                        .to_string_lossy()
                        .replace('\\', "/"),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "`std::env::args()` panics on the first argument that is not UTF-8, which is a core \
         dump under `panic = \"abort\"` and a refused argument for any shim that forwards \
         someone else's argv:\n  {}\n\nRead `std::env::args_os()` and convert where a \
         spelling is actually needed — `to_str()` (fallible) or `to_string_lossy()`.",
        offenders.join("\n  ")
    );
}

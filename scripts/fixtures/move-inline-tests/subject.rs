//! A fixture that holds every shape the mover must survive, and a test that
//! runs the mover over it and asserts what comes out.
//!
//! **The shapes are the knowledge, and this fixture is where the knowledge
//! lives.** Each one is a case a hand-mover got wrong at least once while the
//! sweep ran, and each one is here so the next person who finds a new shape
//! adds it as a case rather than as a memory:
//!
//! 1. a flat module — one test, one file;
//! 2. several modules in one file — the grouping is kept, because two of them
//!    can hold helpers with the same name and different bodies;
//! 3. a helper with an inner `#[cfg(unix)] { }` block;
//! 4. a `format!` argument that runs over two lines;
//! 5. `use super::{A, B}` and `use super::sibling::X` in one module;
//! 6. a `#[cfg(unix)]` gate above an import;
//! 7. a `#[cfg(test)]` constructor that must not ship;
//! 8. a `super::Item` **path** rather than an import;
//! 9. an `include_str!` with a path relative to the src file;
//! 10. raw strings whose braces and quotes are not Rust syntax.

use crate::core::{
    error::Error,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(unix)]
use crate::core::executor::MockExecutor;

pub struct Subject {
    pub(crate) name: String,
    count: usize,
}

impl Subject {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
        }
    }

    /// **`#[cfg(test)]` and it stays that way.** A constructor that must not
    /// ship is the one thing the mover is not allowed to move.
    #[cfg(test)]
    fn with_count(count: usize) -> Self {
        Self {
            name: "test".to_string(),
            count,
        }
    }

    pub fn counted(&mut self) -> usize {
        self.count += 1;
        self.count
    }
}

pub fn sibling_call(subject: &mut Subject) -> usize {
    subject.counted()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The path is relative to the src file** that holds it, which is why the
    /// tool must make it one directory deeper rather than translating it as code.
    const BAKE_ANNOUNCEMENT: &str = include_str!("../../fixtures/move-inline-tests/fixture-output.txt");
    const RAW: &str = r#"{"not_code": "{}"}"#;

    #[test]
    fn a_flat_assertion_names_the_subject_it_was_given() {
        let subject = Subject::new("apt:curl");
        assert_eq!(subject.name, "apt:curl");
    }

    #[test]
    fn an_included_file_is_read_from_where_the_tool_put_it() {
        assert!(BAKE_ANNOUNCEMENT.contains("fixture"));
        assert!(RAW.contains("not_code"));
    }

    #[test]
    fn a_sibling_module_is_reachable_from_the_same_crate() {
        let mut subject = Subject::new("cargo:fd");
        assert_eq!(sibling_call(&mut subject), 1);
    }
}

#[cfg(test)]
mod second_shape_tests {
    use super::*;
    use crate::core::error::Error as TheError;

    fn known(name: &str) -> bool {
        name == "apt" || name == "cargo"
    }

    #[test]
    fn an_inner_cfg_block_keeps_its_shape() {
        #[cfg(unix)]
        let which = "unix";
        #[cfg(windows)]
        let which = "windows";
        assert!(!which.is_empty());
    }

    #[test]
    fn a_format_argument_that_runs_over_two_lines_stays_one_argument() {
        let ticks = 3;
        let line = format!(
            "1..{ticks} | ForEach-Object {{ [Console]::Out.WriteLine('tick'); \
             [Console]::Out.Flush(); Start-Sleep -Milliseconds 500 }}"
        );
        assert!(line.contains("1..3"), "{line}");
    }

    #[test]
    fn a_path_names_the_item_without_importing_it() {
        let err = TheError::command_failed_classified("it broke", Retryability::Permanent);
        assert!(err.to_string().contains("it broke"));
    }
}

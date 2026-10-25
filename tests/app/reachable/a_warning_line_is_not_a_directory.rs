use super::*;

/// A tool answers with a path and sometimes with a paragraph first. Reading the first line
/// would report `warning …` as a directory.
#[test]
fn a_warning_line_is_not_a_directory() {
    let dir = PathBuf::from(SANDBOX).join("yarn-global");
    let noisy = format!(
        "warning package.json: No license field\n{}\n",
        dir.display()
    );
    assert_eq!(first_absolute_path(&noisy), Some(dir));
    assert_eq!(first_absolute_path("warning: nothing to say\n"), None);
}

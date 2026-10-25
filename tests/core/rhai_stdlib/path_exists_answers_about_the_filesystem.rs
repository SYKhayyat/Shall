use super::*;

/// `path_exists` answers about the filesystem rather than about the string, which is the only
/// reason a script would call it.
#[test]
fn path_exists_answers_about_the_filesystem() {
    let engine = engine("test");
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("present.txt");
    std::fs::write(&file, "x").unwrap();

    // The path is bound as a variable rather than interpolated into the script, because a
    // Windows path is mostly backslashes and every one of them would need escaping twice -
    // once for Rust and once for Rhai. A value handed to the scope needs neither.
    let ask = |path: &std::path::Path| -> bool {
        let mut scope = rhai::Scope::new();
        scope.push("p", path.to_string_lossy().to_string());
        engine
            .eval_with_scope(&mut scope, "path_exists(p)")
            .expect("path_exists failed")
    };

    assert!(ask(&file), "a file that exists was reported absent");
    assert!(
        !ask(&dir.path().join("no-such-file")),
        "a file that does not exist was reported present"
    );
}

use super::*;

/// A `.py` provider finds whatever this machine calls Python.
///
/// It used to name literally `python` on Windows and literally `python3` everywhere else, so
/// a Windows box with only `python3` and a Linux box with only `python` each had a `vars.py`
/// they could not run — the same one-spelling assumption that made `#!` hooks fail (Y17).
#[test]
fn a_py_provider_finds_this_machines_python_whatever_it_is_called() {
    let argv = interpreter_argv(Path::new("vars.py"));
    let first = argv
        .first()
        .expect("an interpreter")
        .to_string_lossy()
        .into_owned();
    match shall::model::script::interpreter_named("python3") {
        Some(_) => assert!(
            Path::new(&first).is_file(),
            "python was found but the argv still says `{}`",
            first
        ),
        None => assert_eq!(first, "python3", "an unresolvable name is left as written"),
    }
}

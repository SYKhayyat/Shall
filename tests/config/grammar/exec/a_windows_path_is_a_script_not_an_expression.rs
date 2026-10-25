use super::*;

/// A path with punctuation is a path, not set math — the same rule that keeps a Windows
/// `link:` target from being read as a difference.
#[test]
fn a_windows_path_is_a_script_not_an_expression() {
    let Statement::Exec(script, _) = pv(r"exec:C:\Users\me\bin\setup.ps1").unwrap() else {
        panic!("not an exec");
    };
    assert_eq!(script, r"C:\Users\me\bin\setup.ps1");
}

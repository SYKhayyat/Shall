//! **A PowerShell-level failure must not exit 0.**
//!
//! `exit $LASTEXITCODE` propagates a *native* process's code and nothing else; the command
//! not being found, the `.ps1` being gone, an exception thrown all leave it `$null`, and
//! `exit $null` exits 0 — which `ensure_status` reads as a successful install. The catch is
//! what covers those, and `$LASTEXITCODE` still decides whenever the tool actually ran.

#[cfg(windows)]
#[test]
fn a_powershell_level_failure_is_not_reported_as_success() {
    let (_prog, args) = shall::core::launch::windows_shim_wrap(
        "scoop",
        std::path::Path::new(r"C:\s.ps1"),
        &["install".to_string()],
    )
    .unwrap();
    let command = args.last().unwrap();
    assert!(command.contains("catch"), "no catch: {command}");
    assert!(
        command.contains("exit 1"),
        "the catch must exit non-zero: {command}"
    );
    assert!(
        !command.contains("exit $LASTEXITCODE"),
        "the bare form is the defect: {command}"
    );
    // Not `Stop`: the tool's own non-terminating errors are the tool's business, and scoop
    // emits them on installs that succeed.
    assert!(!command.contains("ErrorActionPreference"), "{command}");
}

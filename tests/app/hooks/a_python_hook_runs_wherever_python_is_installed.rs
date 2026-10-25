use super::*;

/// A Python hook, which is the case the ruling was about.
///
/// Windows has no shebang mechanism: this hook used to reach `CreateProcess` as a bare file
/// and come back *"not a valid application for this OS platform"* — a message about the
/// script, for a failure of the platform. `python3` is also the name Unix uses and a Windows
/// machine usually spells `python`, so the lookup tries both, and `py` after them.
#[tokio::test]
async fn a_python_hook_runs_wherever_python_is_installed() {
    let installed = ["python3", "python", "py"]
        .iter()
        .any(|name| shall::core::launch::resolve_program(name).is_some());
    if !installed {
        // Not silent: a skipped test that reads as a pass is how a dead path survives.
        eprintln!("skipped: no python on this machine (tried python3, python, py)");
        return;
    }

    let dir = tempfile::tempdir().expect("a temp dir");
    let marker = dir.path().join("ran.txt");
    let hooks = hooks_with(&format!(
        "#!/usr/bin/env python3\nimport os\nopen(r'{}', 'w').write(os.environ['SHALL_PKG_NAME'])\n",
        marker.to_string_lossy().replace('\\', "/")
    ));

    hooks
        .run_hook("after_install", "pkg")
        .await
        .expect("the hook ran");
    let wrote = std::fs::read_to_string(&marker).expect("python wrote its marker");
    assert_eq!(wrote.trim(), "pkg");
}

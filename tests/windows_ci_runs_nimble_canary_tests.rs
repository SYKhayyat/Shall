//! The checked-in Windows job must actually make Nimble available to the native sweep.
//!
//! This is a source-level control because the sweep needs a Windows runner and a real Nim
//! compiler. It checks the workflow's setup, index refresh, shell, binary path, and the exact
//! backend/package arguments against the canary contract in the sweep itself.

use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

fn windows_native_job() -> String {
    let ci = read(".github/workflows/ci.yml");
    let start = ci
        .find("\n  windows-native:\n")
        .map(|offset| offset + 1)
        .expect("ci.yml must retain the windows-native job");
    let rest = &ci[start..];
    let end = rest
        .find("\n  argv-drift:\n")
        .expect("windows-native must remain before argv-drift");
    rest[..end].to_string()
}

#[test]
fn windows_native_job_sets_up_and_runs_the_nimble_canary() {
    let job = windows_native_job();
    let sweep = read("scripts/integration-windows.sh");

    assert!(job.contains("runs-on: windows-latest"));
    assert!(job.contains("uses: jiro4989/setup-nim-action@v2"));
    assert!(job.contains("nim-version: '2.2.12'"));
    assert!(job.contains("repo-token: ${{ github.token }}"));
    assert!(job.contains(
        "- name: Refresh Nimble package index\n      shell: powershell\n      run: nimble -y refresh"
    ));
    assert!(job.contains(
        "- name: Native sweep (nimble)\n      shell: bash\n      run: SHALL=\"$PWD/target/release/shall.exe\" bash scripts/integration-windows.sh nimble nimjson"
    ));

    let setup = job
        .find("uses: jiro4989/setup-nim-action@v2")
        .expect("Nim setup must be present");
    let refresh = job
        .find("run: nimble -y refresh")
        .expect("Nimble's package index must be refreshed");
    let nimble_sweep = job
        .find("Native sweep (nimble)")
        .expect("the Windows job must invoke the Nimble sweep");
    assert!(setup < refresh && refresh < nimble_sweep);

    assert!(sweep.contains("scripts/integration-windows.sh [backend] [package]"));
    assert!(sweep.contains("nimble)   echo \"nimjson|nimjson|full|\""));
}

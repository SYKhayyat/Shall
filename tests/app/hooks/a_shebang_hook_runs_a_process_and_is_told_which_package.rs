use super::*;

/// A `#!` hook runs a real process, and the four facts reach it as environment variables.
///
/// The interpreter here is whichever shell the platform is certain to have, so this covers
/// the mechanism on every machine; the Python test beside it covers the case that made the
/// mechanism necessary.
#[tokio::test]
async fn a_shebang_hook_runs_a_process_and_is_told_which_package() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let marker = dir.path().join("ran.txt");
    let hooks = hooks_with(&if cfg!(windows) {
        format!(
            "#!/usr/bin/env powershell\nSet-Content -Path '{}' -Value $env:SHALL_PKG_NAME\n",
            marker.display()
        )
    } else {
        format!(
            "#!/bin/sh\nprintf %s \"$SHALL_PKG_NAME\" > '{}'\n",
            marker.display()
        )
    });

    hooks
        .run_hook("after_install", "pkg")
        .await
        .expect("the hook ran");
    let wrote = std::fs::read_to_string(&marker).expect("the hook wrote its marker");
    assert_eq!(wrote.trim(), "pkg");
}

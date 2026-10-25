use super::*;

/// The shim spawning itself, closed.
///
/// `bin_dir` sits on `PATH` ahead of the real binary — that is what makes a shim a shim — so
/// resolving the shimmed name by bare name found the shim, which re-entered Shall, which
/// resolved the name again. Nothing in the tree stopped it: no depth counter, no marker,
/// no exclusion.
#[tokio::test]
async fn a_shim_on_path_is_never_what_the_runner_spawns() {
    let tmp = tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    let real_dir = tmp.path().join("usr-bin");
    tokio::fs::create_dir_all(&real_dir).await.unwrap();
    let real = real_dir.join(deployed_name("jq"));
    tokio::fs::write(&real, b"the real jq").await.unwrap();

    // The control: nothing is a shim yet, so the first PATH entry wins as it always has.
    let decoy = bin_dir.join(deployed_name("jq"));
    tokio::fs::create_dir_all(&bin_dir).await.unwrap();
    tokio::fs::write(&decoy, b"someone else's jq")
        .await
        .unwrap();
    let path = std::env::join_paths([&bin_dir, &real_dir]).unwrap();
    assert_eq!(
        real_program_on("jq", Some(path.clone())).await,
        decoy.to_string_lossy(),
        "a file Shall did not deploy is a normal binary and must still be found first"
    );

    // And now the same PATH with a real shim in front of the real binary.
    tokio::fs::remove_file(&decoy).await.unwrap();
    ShimManager::with_bin_dir(bin_dir.clone())
        .await
        .unwrap()
        .create_shim("jq")
        .await
        .unwrap();
    assert!(decoy.exists(), "the fixture needs a deployed shim to skip");
    assert_eq!(
        real_program_on("jq", Some(path)).await,
        real.to_string_lossy(),
        "the runner resolved the shimmed name back to the shim — that is the recursion"
    );
}

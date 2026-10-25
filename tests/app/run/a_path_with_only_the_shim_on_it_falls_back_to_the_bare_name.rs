use super::*;

/// A `PATH` holding nothing but the shim hands the bare name to the OS, so the user gets the
/// error they would have got by typing it rather than a silent re-entry.
#[tokio::test]
async fn a_path_with_only_the_shim_on_it_falls_back_to_the_bare_name() {
    let tmp = tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    ShimManager::with_bin_dir(bin_dir.clone())
        .await
        .unwrap()
        .create_shim("jq")
        .await
        .unwrap();
    let path = std::env::join_paths([&bin_dir]).unwrap();
    assert_eq!(real_program_on("jq", Some(path)).await, "jq");
}

use super::*;

#[test]
fn tar_gz_round_trips_through_extract() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("bundle");
    std::fs::create_dir_all(src.join("modules")).unwrap();
    std::fs::write(src.join("modules/dev.txt"), "apt:curl\n").unwrap();
    std::fs::write(src.join("packages.json"), "{}").unwrap();

    let tar = tmp.path().join("bundle.tar.gz");
    let size = shall::utils::archive::create_tar_gz(&src, &tar, "bundle").unwrap();
    assert!(size > 0);
    assert!(tar.exists());

    // Unpack it and confirm the tree survived under the single root folder.
    let dest = tmp.path().join("unpacked");
    shall::utils::archive::extract_archive(&tar, &dest).unwrap();
    let restored = dest.join("bundle/modules/dev.txt");
    assert_eq!(std::fs::read_to_string(restored).unwrap(), "apt:curl\n");
}

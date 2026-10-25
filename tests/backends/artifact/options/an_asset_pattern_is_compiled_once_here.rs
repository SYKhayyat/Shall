use super::*;

#[test]
fn an_asset_pattern_is_compiled_once_here() {
    let read = ArtifactOptions::read(&opts(&[("asset", "*musl*")])).unwrap();
    assert!(read.asset.unwrap().matches("tool-musl.deb"));
}

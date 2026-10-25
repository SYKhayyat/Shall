use super::*;

#[test]
fn an_explicit_target_beats_a_shorter_silent_one() {
    let formats = FormatOrder::new(vec![Format::Tarball]);
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd.tar.gz", "fd-linux-x86_64.tar.gz"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd-linux-x86_64.tar.gz");
}

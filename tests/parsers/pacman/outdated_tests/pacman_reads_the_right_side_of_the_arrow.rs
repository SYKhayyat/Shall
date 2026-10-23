use super::*;

#[test]
fn pacman_reads_the_right_side_of_the_arrow() {
    let p = parse_pacman_outdated(PACMAN);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "archlinux-keyring");
    assert_eq!(p[0].version.as_deref(), Some("20260727-1"));
    // A version containing `+r581+g...` must survive intact — it is arch's own shape for
    // a VCS build and splitting it differently loses the identity.
    assert_eq!(p[2].version.as_deref(), Some("16.1.1+r595+g171d15ac6959-1"));
}

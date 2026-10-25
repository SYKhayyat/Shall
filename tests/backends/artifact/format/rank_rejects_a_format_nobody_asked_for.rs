use super::*;

#[test]
fn rank_rejects_a_format_nobody_asked_for() {
    let order = FormatOrder::new(vec![Format::Deb, Format::Tarball]);
    assert_eq!(order.rank(Format::Deb), Some(0));
    assert_eq!(order.rank(Format::Tarball), Some(1));
    assert_eq!(order.rank(Format::Rpm), None);
}

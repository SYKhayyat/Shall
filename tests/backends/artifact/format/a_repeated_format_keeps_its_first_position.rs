use super::*;

#[test]
fn a_repeated_format_keeps_its_first_position() {
    let order = FormatOrder::new(vec![Format::Deb, Format::Tarball, Format::Deb]);
    assert_eq!(order.as_slice(), &[Format::Deb, Format::Tarball]);
}

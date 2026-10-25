use super::*;

#[test]
fn a_plain_list_keeps_its_order() {
    let p = parse("apt\ndnf\ncargo\nsnap\n").unwrap();
    assert_eq!(p.order(), ["apt", "dnf", "cargo", "snap"]);
}

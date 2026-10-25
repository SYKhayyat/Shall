use super::*;

#[test]
fn a_backend_named_twice_keeps_its_first_position() {
    let p = parse("when host == laptop {\n  apt\n}\ncargo\napt\n").unwrap();
    assert_eq!(p.order(), ["apt", "cargo"]);
}

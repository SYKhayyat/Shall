use super::*;

#[test]
fn shim_carries_its_source() {
    let Statement::Shim(name, opts) = p("shim:jq@source=cargo:jq").unwrap() else {
        panic!()
    };
    assert_eq!(name, "jq");
    assert_eq!(opts.one("source"), Some("cargo:jq"));
}

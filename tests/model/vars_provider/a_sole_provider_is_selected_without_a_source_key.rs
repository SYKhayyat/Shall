use super::*;

#[test]
fn a_sole_provider_is_selected_without_a_source_key() {
    let tmp = tmp();
    let dir = tmp.path();
    std::fs::write(dir.join("vars"), "role = desktop\n").unwrap();
    let s = select(dir, &None).unwrap().unwrap();
    assert_eq!(s.kind, Kind::LineFile);
}

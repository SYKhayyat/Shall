use super::*;

#[test]
fn a_named_source_picks_that_one_and_names_its_kind() {
    let tmp = tmp();
    let dir = tmp.path();
    std::fs::write(dir.join("vars"), "role = desktop\n").unwrap();
    std::fs::write(dir.join("vars.py"), "print()\n").unwrap();
    let s = select(dir, &Some("vars.py".into())).unwrap().unwrap();
    assert_eq!(s.kind, Kind::External);
    assert_eq!(s.path, dir.join("vars.py"));
}

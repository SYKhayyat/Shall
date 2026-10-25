use super::*;

#[test]
fn statements_carry_where_they_came_from() {
    // II.7 rule 5 needs this: a conflict must name BOTH files.
    let d = doc("# header\napt:curl\n").unwrap();
    let out = d.statements_for(&facts()).unwrap();
    assert_eq!(out[0].1.line, 2);
    assert_eq!(out[0].1.file, PathBuf::from("modules/dev.txt"));
}

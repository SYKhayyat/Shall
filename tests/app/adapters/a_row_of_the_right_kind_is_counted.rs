use super::*;

#[test]
fn a_row_of_the_right_kind_is_counted() {
    let body = "[[backend]]\nname = \"a\"\n\n[[backend]]\nname = \"b\"\n";
    assert_eq!(
        standing(&[("backends", body)], "backends"),
        Standing::InUse { rows: 2 }
    );
    assert!(standing(&[("backends", body)], "backends").effective());
}

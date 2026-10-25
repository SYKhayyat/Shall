use super::*;

#[test]
fn a_block_value_keeps_a_hash_that_is_part_of_the_value() {
    let (_, v) = parse_block_line(&o(), r#"after_install = curl -H "X:#tag""#).unwrap();
    assert_eq!(v, r#"curl -H "X:#tag""#);
}

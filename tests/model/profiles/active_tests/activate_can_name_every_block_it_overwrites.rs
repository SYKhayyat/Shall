use super::*;

#[test]
fn activate_can_name_every_block_it_overwrites() {
    let blocks = blocks_in_active("Work\nwhen host == laptop {\n  Travel\n}\n");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].predicate, "host == laptop");
    assert_eq!(blocks[0].line, 2);
}

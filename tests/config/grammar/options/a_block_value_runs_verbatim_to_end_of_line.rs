use super::*;

#[test]
fn a_block_value_runs_verbatim_to_end_of_line() {
    let (k, v) = parse_block_line(&o(), "  after_install = ./setup.sh --flag=a,b  ").unwrap();
    assert_eq!(k, "after_install");
    // Commas and `=` are data here — that is the whole point of the block form.
    assert_eq!(v, "./setup.sh --flag=a,b");
}

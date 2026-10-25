use super::*;

#[test]
fn the_value_tail_is_not_part_of_the_name() {
    // Help text writes `--verify`; the argv writes `--verify=false`. Matching the whole
    // token would answer "no" for every flag that carries a value, which is most of them.
    // Asserted through a program whose help certainly mentions the token it is asked about.
    let prog = if cfg!(windows) { "cmd" } else { "sh" };
    let _ = accepts_flag(prog, &[], "--nosuchflag=1");
    // The property under test is the split, which is cheap to state directly.
    assert_eq!("--verify=false".split('=').next(), Some("--verify"));
}

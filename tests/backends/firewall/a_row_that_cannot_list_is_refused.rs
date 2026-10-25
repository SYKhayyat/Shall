use super::*;

/// A row that cannot list its rules is refused: drift it cannot see is drift it would
/// silently never correct, and N7 turns on seeing it.
#[test]
fn a_row_that_cannot_list_is_refused() {
    let mut blind = shipped("ufw");
    blind.name = "blind".into();
    blind.list = vec![];
    assert!(!adapters(vec![blind]).iter().any(|a| a.name == "blind"));
}

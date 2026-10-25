use super::*;

/// A machine dpkg knows only removed packages on is a machine with nothing installed — not
/// a listing the parser failed to read. The two are different answers and reading the first
/// as the second would refuse to run on it.
#[test]
fn every_row_removed_is_an_empty_machine_and_not_an_unread_one() {
    let res = parse_list("config-files figlet 2.2.5-3\nnot-installed sl \n")
        .expect("rows that were read and dropped are an answer");
    assert!(res.is_empty());
}

use super::*;

#[test]
fn a_gated_name_is_in_the_file_and_says_which_block_holds_it() {
    // What `deactivate` needs to say "it is still activated by the `when` block on
    // line 4" rather than silently doing nothing.
    let entries = read(II6_EXAMPLE, "laptop").unwrap();
    let travel = entries.iter().find(|e| e.name == "Travel").unwrap();
    assert_eq!(travel.gate.as_deref(), Some("host == laptop"));
    assert_eq!(travel.line, 5);
    assert!(travel.on);

    // On another host it is still in the file, just not in force.
    let entries = read(II6_EXAMPLE, "server").unwrap();
    let travel = entries.iter().find(|e| e.name == "Travel").unwrap();
    assert!(!travel.on);
}

use super::*;

/// The line the daemon prints: five names, then the count, because a pull that brings forty
/// files is one event and one line.
#[test]
fn the_line_names_what_it_reacted_to_without_becoming_a_list() {
    assert_eq!(say(&[]), "nothing changed");
    assert_eq!(say(&["active".to_string()]), "active");
    assert_eq!(
        say(&["a".to_string(), "b".to_string()]),
        "a, b",
        "two names fit on one line"
    );
    let six: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(say(&six), "a, b, c, d, e and 2 more");
}

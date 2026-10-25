use super::*;

#[test]
fn genuinely_empty_output_is_an_empty_machine() {
    // The control, without which the assertion above holds for a parser that errors on
    // everything: a manager with nothing installed prints nothing, and that is an answer.
    assert!(parse_pkg("").expect("no output is no packages").is_empty());
    assert!(parse_pkg("\n  \n").expect("blank output").is_empty());
}

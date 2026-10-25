use super::*;

#[test]
fn a_carriage_return_is_not_part_of_the_interpreter() {
    // A config edited on Windows and committed without translation reaches here as CRLF. A
    // `python3\r` resolves to nothing and reports a missing interpreter naming an invisible
    // character.
    let found = shebang("#!/usr/bin/env python3\r\nprint(1)\r\n").expect("a shebang");
    assert_eq!(found.interpreter, "python3");
}

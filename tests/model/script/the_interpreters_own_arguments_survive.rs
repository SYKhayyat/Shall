use super::*;

#[test]
fn the_interpreters_own_arguments_survive() {
    let found = shebang("#!/bin/bash -e\n").expect("a shebang");
    assert_eq!(found.interpreter, "/bin/bash");
    assert_eq!(found.args, ["-e"]);
}

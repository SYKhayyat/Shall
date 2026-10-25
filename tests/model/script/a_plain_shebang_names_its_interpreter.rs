use super::*;

#[test]
fn a_plain_shebang_names_its_interpreter() {
    let found = shebang("#!/bin/bash\necho hi\n").expect("a shebang");
    assert_eq!(found.interpreter, "/bin/bash");
    assert!(found.args.is_empty());
}

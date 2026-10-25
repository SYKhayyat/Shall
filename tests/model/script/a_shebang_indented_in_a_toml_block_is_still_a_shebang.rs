use super::*;

#[test]
fn a_shebang_indented_in_a_toml_block_is_still_a_shebang() {
    // The same leading newline and indentation the `#rhai` arm already tolerates, because
    // TOML multi-line strings routinely arrive that way. A kernel would refuse this file;
    // the interpreter is named here instead, so it runs.
    let found = shebang("\n  #!/usr/bin/env python3\n  print(1)\n").expect("a shebang");
    assert_eq!(found.interpreter, "python3");
}

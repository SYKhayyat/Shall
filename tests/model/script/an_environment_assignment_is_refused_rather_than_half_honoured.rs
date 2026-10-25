use super::*;

#[test]
fn an_environment_assignment_is_refused_rather_than_half_honoured() {
    let err = shebang_of("#!/usr/bin/env -S FOO=1 python3\n").expect_err("refused");
    assert!(err.to_string().contains("FOO=1"), "{}", err);
    assert!(err.to_string().contains("inside the script"), "{}", err);
}

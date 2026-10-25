use super::*;

#[test]
fn an_exec_names_a_script() {
    let Statement::Exec(script, opts) = pv("exec:./bin/enroll-tpm.sh").unwrap() else {
        panic!("not an exec");
    };
    assert_eq!(script, "./bin/enroll-tpm.sh");
    assert!(opts.one("runs").is_none(), "no ceiling means the default");
}

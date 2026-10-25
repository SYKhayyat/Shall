use super::*;

#[test]
fn an_exec_takes_a_runs_ceiling() {
    let Statement::Exec(_, opts) = pv("exec:./setup.sh@runs=3").unwrap() else {
        panic!("not an exec");
    };
    assert_eq!(opts.one("runs"), Some("3"));
    let Statement::Exec(_, opts) = pv("exec:./tick.sh@runs=always").unwrap() else {
        panic!("not an exec");
    };
    assert_eq!(opts.one("runs"), Some("always"));
}

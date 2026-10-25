use super::*;

#[test]
fn a_generate_names_a_command() {
    let Statement::Generate(cmd, _) = pv("generate:./bin/pick.sh").unwrap() else {
        panic!("not a generate");
    };
    assert_eq!(cmd, "./bin/pick.sh");
    assert_eq!(
        pv("generate:./bin/pick.sh").unwrap().kind(),
        Some(ResourceKind::Generate)
    );
}

use super::*;

#[test]
fn a_program_that_is_not_here_answers_nothing_rather_than_no() {
    // `None`, not `Some(false)`. A tool that is absent has said nothing about its flags,
    // and treating silence as a refusal is what broke two unit tests on every CI runner
    // while passing on the one machine that happened to have helm.
    assert_eq!(
        accepts_flag(
            "shall-no-such-program-zzz",
            &["plugin".into(), "install".into()],
            "--verify=false"
        ),
        None
    );
}

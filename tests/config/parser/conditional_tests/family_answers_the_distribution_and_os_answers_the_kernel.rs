use super::*;

#[test]
fn family_answers_the_distribution_and_os_answers_the_kernel() {
    let f = facts();
    assert!(eval_when("family == debian", &f).unwrap());
    assert!(eval_when("os == linux", &f).unwrap());
    assert!(
        !eval_when("family == linux", &f).unwrap(),
        "`family` must not still be answering the os question"
    );
}

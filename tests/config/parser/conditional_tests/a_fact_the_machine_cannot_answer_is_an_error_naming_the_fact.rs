use super::*;

#[test]
fn a_fact_the_machine_cannot_answer_is_an_error_naming_the_fact() {
    // `home` unanswerable is not the same failure as `kernel` unknown: the first says
    // what could not be detected, the second is a typo.
    let mut f = facts();
    f.home = None;
    f.user = None;
    for key in ["home", "user"] {
        let err = eval_when(&format!("{} == x", key), &f).unwrap_err();
        assert!(
            err.to_string().contains("cannot say"),
            "an undetectable fact must say so: {}",
            err
        );
    }
    assert!(eval_when("kernel == 6.1", &f).is_err());
}

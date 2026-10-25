use super::*;

#[test]
fn home_and_user_are_facts_the_machine_reports() {
    // #69: the per-user substitution vocabulary. Facts, not variables — so they are
    // reached bare, and a `vars` entry of the same name does not move them.
    let f = facts();
    assert!(eval_when("home == /home/u", &f).unwrap());
    assert!(!eval_when("home == /root", &f).unwrap());
    assert!(eval_when("user == u", &f).unwrap());
    assert!(!eval_when("user == root", &f).unwrap());

    let mut vars = shall::model::vars::Vars::new();
    vars.insert("home".to_string(), Value::Str("/elsewhere".into()));
    let g = facts().with_vars(vars);
    assert!(
        eval_when("home == /home/u", &g).unwrap(),
        "a variable of the same name must not move the fact (IX.4)"
    );
    assert!(eval_when("$home == /elsewhere", &g).unwrap());
}

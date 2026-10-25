use super::*;

#[test]
fn a_when_block_in_active_can_test_a_variable() {
    // W8: `active` is the most useful place for a variable, and it used to fail with
    // "unknown when key '$role'" because it detected its own varless facts.
    let mut f = facts("anyhost");
    f.vars.insert(
        "role".into(),
        shall::model::vars::Value::Str("travel".into()),
    );
    let body = "when $role == travel {\n  Travel\n}\nWork\n";
    let names: Vec<String> = read_active(&PathBuf::from("active"), body, &f)
        .unwrap()
        .into_iter()
        .filter(|e| e.on)
        .map(|e| e.name)
        .collect();
    assert_eq!(names, vec!["Travel".to_string(), "Work".to_string()]);

    f.vars.insert(
        "role".into(),
        shall::model::vars::Value::Str("desktop".into()),
    );
    let names: Vec<String> = read_active(&PathBuf::from("active"), body, &f)
        .unwrap()
        .into_iter()
        .filter(|e| e.on)
        .map(|e| e.name)
        .collect();
    assert_eq!(
        names,
        vec!["Work".to_string()],
        "Travel is off when the variable does not match"
    );
}

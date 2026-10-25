use super::*;

#[test]
fn a_variable_block_chooses_a_backend() {
    let mut f = facts();
    f.vars.insert(
        "role".into(),
        shall::model::vars::Value::parse_literal("travel"),
    );
    let p = Priority::parse(
        &PathBuf::from("priority"),
        "apt\nwhen $role == travel {\n  cargo\n}\n",
        &f,
    )
    .unwrap();
    assert_eq!(p.order(), ["apt", "cargo"]);
}

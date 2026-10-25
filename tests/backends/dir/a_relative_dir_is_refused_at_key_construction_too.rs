use super::*;

#[test]
fn a_relative_dir_is_refused_at_key_construction_too() {
    for declared in ["relative/state", "."] {
        let error = shall::core::extras_lock::extra_key(&shall::config::grammar::Statement::Dir(
            declared.to_string(),
            Default::default(),
        ))
        .expect_err("a relative dir: has no stable key")
        .to_string();
        assert!(error.contains(&format!("dir:{declared}")), "{error}");
        assert!(error.contains("not an absolute path"), "{error}");
    }
}

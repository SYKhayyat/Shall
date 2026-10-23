use super::*;

#[test]
fn the_suppression_overrides_a_user_pager() {
    let mut env = HashMap::new();
    env.insert("PAGER".to_string(), "less -R".to_string());
    CommandExecutor::suppress_pagers(&mut env);
    assert_eq!(env.get("PAGER").map(String::as_str), Some("cat"));
}

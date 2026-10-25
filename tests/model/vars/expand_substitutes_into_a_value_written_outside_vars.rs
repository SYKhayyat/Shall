use super::*;

#[test]
fn expand_substitutes_into_a_value_written_outside_vars() {
    let mut facts = expand_facts();
    facts.vars.insert("role".to_string(), str_val("travel"));
    let out = expand("~/.config/$role/init.lua", &facts, &origin(3)).unwrap();
    assert_eq!(out, "~/.config/travel/init.lua");
}

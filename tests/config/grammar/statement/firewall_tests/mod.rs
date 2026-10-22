use super::*;

pub use shall::config::grammar::statement::*;
fn o() -> Origin {
    Origin::new("modules/net.txt", 2)
}
fn known(name: &str) -> bool {
    matches!(name, "apt")
}
fn pv(line: &str) -> Result<Statement> {
    let s = parse(&o(), line, &known)?;
    validate(&o(), &s)?;
    Ok(s)
}

mod a_default_policy_needs_a_direction_and_a_value;
mod a_firewall_rule_is_an_extra_with_a_teardown_key;
mod a_port_rule_is_its_own_whole_declaration;
mod a_port_rule_refuses_a_value;
mod a_rule_the_grammar_cannot_read_is_refused_at_parse_time;

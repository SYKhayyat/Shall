pub use shall::model::firewall::*;
use std::fmt;

fn port(p: u16) -> Rule {
    Rule::Port {
        port: p,
        proto: Proto::Tcp,
    }
}
mod a_change_that_leaves_the_session_alone_is_permitted;
mod a_default_policy_parses_in_both_directions;
mod a_port_and_protocol_parse;
mod a_rule_round_trips_through_display;
mod anything_else_is_refused_with_a_reason;
mod removing_the_session_port_is_refused;
mod the_refusal_names_the_port_and_the_way_out;
mod tightening_the_default_closes_the_session_port_without_naming_it;
mod with_no_session_port_nothing_is_refused;

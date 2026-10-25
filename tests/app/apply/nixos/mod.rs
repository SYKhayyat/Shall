pub use shall::app::apply::nixos::*;
pub use shall::config::grammar::{Options, Origin, Statement};
pub use shall::core::{Error, Result};
pub use shall::model::firewall::{Direction, Proto, Rule};
pub use std::collections::{BTreeMap, BTreeSet};
use tracing::info;

fn origin() -> Origin {
    Origin::new("modules/test.txt", 1)
}
fn opts(pairs: &[(&str, &str)]) -> Options {
    let mut o = Options::default();
    for (k, v) in pairs {
        o.set(*k, *v);
    }
    o
}
fn state(extras: Vec<Statement>) -> shall::model::DesiredState {
    shall::model::DesiredState {
        extras: extras.into_iter().map(|s| (s, origin())).collect(),
        ..Default::default()
    }
}
mod a_contradictory_line_leaves_the_refusal_to_the_projection;
mod a_line_this_os_cannot_express_is_refused_by_name;
mod a_state_with_neither_declares_nothing_and_says_so;
mod an_allow_default_turns_the_firewall_off_rather_than_leaving_it_unsaid;
mod one_service_declared_two_ways_is_refused_rather_than_settled_by_order;
mod only_a_restart_is_left_for_the_init_to_do;
mod services_and_the_perimeter_become_nixos_attributes;

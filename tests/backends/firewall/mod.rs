use serde::Deserialize;
pub use shall::backends::firewall::*;
pub use shall::core::adapter::{self, AdapterRow, Detected};
pub use shall::model::firewall::{Direction, Proto, Rule};

fn shipped(name: &str) -> FirewallAdapter {
    adapters(vec![])
        .into_iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| panic!("{} must ship", name))
}
mod a_default_policy_becomes_the_firewalls_own_verb;
mod a_direction_a_firewall_cannot_express_is_not_pretended;
mod a_row_that_cannot_list_is_refused;
mod an_empty_or_noisy_listing_yields_no_rules;
mod closing_a_port_is_its_own_command;
mod detection_picks_the_firewall_that_is_present;
mod firewalld_ports_parse;
mod one_declaration_becomes_each_firewalls_own_command;
mod the_shipped_table_parses_and_carries_three_firewalls;
mod ufw_status_parses_only_the_rules;

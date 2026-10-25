pub use shall::model::health::*;

fn one_check() -> Vec<Check> {
    vec![Check {
        subject: "apt:nginx".into(),
        probe: Probe::Port(80),
    }]
}
mod a_bare_string_is_a_command;
mod a_dry_run_never_refuses;
mod a_failure_with_a_snapshot_reverts;
mod a_failure_without_a_snapshot_does_not_pretend_to_revert;
mod a_malformed_port_is_not_quietly_a_command;
mod a_port_probe_parses_to_a_port;
mod a_probe_round_trips_through_its_display;
mod a_provider_makes_the_same_checks_fine;
mod an_empty_probe_is_nothing;
mod declared_checks_with_no_way_back_refuse_before_starting;
mod no_checks_means_no_refusal_either_way;
mod nothing_failing_is_healthy;
mod the_refusal_does_not_repeat_a_subject;
mod the_refusal_names_the_checks_and_the_reason;
mod the_revert_message_says_it_is_restoring_the_snapshot;

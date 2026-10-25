pub use shall::app::sync::resolver::StateResolver;
pub use shall::core::{exit_policy, Error, Retryability};
pub use shall::model::Edit;
pub use shall::verbs::packages::*;
pub use shall::verbs::prelude::*;
pub use shall::verbs::sync::{handle_sync, SyncMode};

fn boxed(e: Error) -> anyhow::Error {
    anyhow::Error::new(e)
}
const NEWLINE: &str = "
";
mod a_failure_that_says_the_name_is_absent_is_withdrawn;
mod a_name_wrapped_across_lines_is_still_recognised_as_mentioned;
mod a_permanent_failure_about_a_name_that_exists_never_withdraws;
mod a_permanent_failure_is_not_advised_as_if_nobody_had_looked;
mod a_permanent_failure_is_not_reported_as_unclassified;
mod a_transient_failure_is_not_advised_as_if_it_were_permanent;
mod a_transient_failure_is_not_reported_as_unclassified;
mod a_transient_or_unclassified_failure_keeps_the_line;
mod an_unresolvable_name_is_still_recognised_and_carries_itself;
mod attribution_does_not_spread_to_a_line_the_manager_never_named;
mod each_failure_is_classified_as_the_reason_its_line_stayed;
mod every_reader_sees_through_a_context_chain;
mod no_other_permanent_error_withdraws_a_line;
mod only_an_unclassified_failure_may_suggest_that_a_retry_could_work;
mod the_absent_fact_survives_the_retry_classifier;
mod the_variants_that_withdraw_carry_the_name_they_looked_up;

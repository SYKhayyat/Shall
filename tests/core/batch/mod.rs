pub use shall::core::batch::node_deadline;
use std::time::Duration;
const NODE: Duration = Duration::from_secs(300);
const TOTAL: Duration = Duration::from_secs(3600);

mod a_bigger_batch_never_gets_less_time;
mod no_batch_is_given_a_budget_the_transaction_would_have_ended_first;
mod one_package_gets_the_per_package_budget;
mod the_batch_still_scales_with_its_size_below_the_ceiling;

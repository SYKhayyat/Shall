pub use shall::app::bisect::*;
pub use shall::config::Config;
pub use shall::core::{Error, Result};
pub use tracing::{info, warn};

async fn over(states: &'static [bool]) -> Option<usize> {
    first_bad(states.len(), |i| async move { Ok(states[i]) })
        .await
        .expect("a synthetic oracle does not fail")
}
mod an_oracle_that_could_not_answer_is_not_a_bad_verdict;
mod first_bad_all_bad_is_zero;
mod first_bad_all_good_is_none;
mod first_bad_empty_is_none;
mod first_bad_finds_transition;
mod first_bad_is_logarithmic;

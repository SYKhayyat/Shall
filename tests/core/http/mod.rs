pub use shall::core::http::*;
pub use shall::core::{Error, Result};

fn mine(user_agent: &str) -> usize {
    pool_size_for_test(user_agent)
}
mod a_client_that_would_follow_a_downgrade_is_never_handed_to_one_that_refuses;
mod an_api_client_never_asks_reqwest_for_a_zero_second_timeout;
mod asking_twice_for_one_policy_builds_one_client;
mod sixteen_threads_asking_at_once_build_one_client;
mod the_timeout_is_part_of_the_key;

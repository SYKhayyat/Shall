use dashmap::DashMap;
pub use shall::core::installed::*;
pub use shall::core::{Package, Result};
use std::future::Future;
use std::path::PathBuf;
pub use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
pub use std::time::{Duration, SystemTime};

fn pkg(name: &str) -> Package {
    Package::new(name, "test")
}
mod a_failure_is_not_remembered_as_an_answer;
mod a_listing_survives_a_run_only_when_asked_for_and_only_until_it_is_stale;
mod a_listing_taken_before_an_invalidation_is_not_served_after_it;
mod a_manager_is_asked_once_however_often_it_is_listed;
mod a_mutation_makes_the_next_listing_real;
mod a_narrow_forget_deletes_the_family_s_files_and_leaves_the_others;
mod a_narrow_forget_drops_the_whole_lock_family_and_nothing_else;
mod a_narrow_forget_still_drops_the_essential_set;
mod only_a_command_that_just_reports_may_be_answered_from_disk;
mod the_essential_set_is_asked_once_and_forgotten_with_the_listings;
mod two_askers_at_once_still_produce_one_fetch;
mod two_managers_do_not_share_one_answer;

mod a_mutation_forgets_only_what_it_could_have_changed_tests;

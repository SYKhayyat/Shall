pub use shall::core::exec_lock::Ceiling;
pub use shall::core::hook_lock::Verdict;
pub use shall::model::exec::*;

fn changed() -> Verdict {
    Verdict::Changed {
        was: "a".repeat(64),
        now: "b".repeat(64),
    }
}
mod a_line_belongs_to_the_verb_it_names_and_a_silent_one_belongs_to_sync;
mod a_short_hash_does_not_panic;
mod always_runs_every_time;
mod an_approved_script_below_its_ceiling_runs;
mod approval_is_checked_before_the_count;
mod the_description_carries_the_hash_the_count_and_the_verdict;
mod the_same_content_does_not_run_twice_by_default;

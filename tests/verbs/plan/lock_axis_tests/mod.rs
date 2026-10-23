pub use shall::app::sync::resolver::StateResolver;
pub use shall::core::lock_kind::{self, LockKind, LockSelection};
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::plan::*;
pub use shall::verbs::prelude::*;
pub use shall::verbs::sync::{enforce_policy, print_vars_changed};

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}
fn pins(entries: &[(&str, &str)]) -> serde_json::Map<String, Value> {
    entries
        .iter()
        .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
        .collect()
}
mod a_class_and_a_name_that_spell_the_same_word_stay_different_questions;
mod a_key_with_no_prefix_matches_itself_and_nothing_else;
mod a_pin_follows_the_package_that_moved;
mod a_pin_the_scan_could_not_read_keeps_its_version;
mod a_scope_matches_the_whole_key_or_its_tail;
mod an_empty_scope_takes_everything;
mod an_unpinned_package_gains_no_pin;
mod any_of_the_names_selects;
mod re_recording_an_already_current_pin_is_not_a_change;
mod the_empty_scope_warning_describes_whichever_scope_was_given;

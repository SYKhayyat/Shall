pub use shall::core::lock_kind::*;
pub use shall::core::{Error, Result};
use std::fmt;

fn sel(what: &str) -> LockSelection {
    LockSelection::parse(what, &[]).expect("should parse")
}
fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}
fn kinds_of(s: &LockSelection) -> Vec<LockKind> {
    let mut k: Vec<LockKind> = s
        .targets()
        .iter()
        .filter(|t| !t.negated)
        .map(|t| t.kind)
        .collect();
    k.sort();
    k.dedup();
    k
}
mod a_flat_kind_refuses_a_qualifier_and_says_what_to_type_instead;
mod a_qualifier_on_a_group_narrows_only_the_members_that_divide;
mod a_qualifier_selects_one_sub_category_and_not_its_neighbours;
mod a_qualifier_that_is_empty_is_refused;
mod a_selection_that_comes_out_empty_is_an_error_rather_than_a_quiet_no_op;
mod a_set_can_be_listed_or_named_as_a_group;
mod an_event_qualifier_matches_the_event_or_where_it_is_declared;
mod an_exclusion_can_name_a_sub_category_and_keep_the_rest_of_the_kind;
mod an_exclusion_subtracts_from_whatever_was_named;
mod an_unknown_word_is_refused_with_the_list_of_real_ones;
mod an_unqualified_kind_admits_all_of_its_entries;
mod config_narrows_the_default_but_never_a_named_kind;
mod every_kind_has_one_name_and_no_two_share_it;
mod every_ledger_prefix_is_claimed_by_exactly_one_kind;
mod every_manager_named_on_the_bare_lock_is_reported_for_checking;
mod spacing_and_repetition_do_not_change_the_set;
mod the_groups_partition_every_kind;

pub use shall::app::rebuild::*;
pub use shall::core::PackageSpec;

fn spec(backend: &str, name: &str) -> PackageSpec {
    PackageSpec {
        name: name.to_string(),
        backend: backend.to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: true,
    }
}
fn foundation(b: &str) -> bool {
    matches!(b, "apt" | "dnf" | "pacman")
}
fn all_installed(_: &str, _: &str) -> bool {
    true
}
fn targets(raw: &[&str]) -> Vec<Target> {
    raw.iter()
        .map(|s| Target::parse(s, |b| matches!(b, "apt" | "cargo" | "web")))
        .collect()
}
mod a_backend_missing_from_priority_still_gets_an_order;
mod a_batch_emptied_by_the_guard_is_not_a_batch;
mod a_batch_holds_one_backend_and_the_batches_are_ordered;
mod a_name_with_colons_in_it_survives_the_split;
mod a_package_nobody_declared_is_named_not_ignored;
mod a_protected_package_is_dropped_and_reported;
mod a_qualified_name_selects_only_that_backends_copy;
mod an_absent_line_is_not_rebuilt;
mod an_unknown_prefix_is_part_of_the_name_not_a_backend;
mod backend_scope_takes_that_backend_whole;
mod declared_but_not_installed_is_left_to_sync;
mod the_foundation_goes_first_whatever_priority_says;
mod within_a_tier_priority_decides;

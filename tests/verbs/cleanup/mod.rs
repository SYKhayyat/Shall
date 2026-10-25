pub use shall::app::sync::guard;
pub use shall::app::sync::resolver::StateResolver;
pub use shall::config::Config;
pub use shall::core::state::ManagedPackage;
pub use shall::verbs::cleanup::managed_where_the_crawl_could_see;
pub use shall::verbs::cleanup::*;
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::prelude::*;

fn reads_as_a_mistake(managed: usize, to_remove: usize) -> bool {
    shall::verbs::cleanup::reads_as_a_mistake(&Config::default(), managed, to_remove)
}
fn managed_pkg(backend: &str, name: &str) -> ManagedPackage {
    ManagedPackage {
        name: name.into(),
        backend: backend.into(),
        version: None,
        installed_at: 0,
        expires_at: None,
        options: Default::default(),
        source: "test".into(),
        is_transient: false,
        session_id: None,
    }
}
mod a_manager_the_crawl_never_asked_is_on_neither_side;
mod an_adopted_machine_may_purge_the_rest;
mod every_manager_that_answered_counts;
mod manage_three_delete_576_is_a_mistake_at_any_scale;
mod managing_nothing_is_always_a_mistake;
mod the_macos_nightly_refuses_once_both_sides_count_the_same_machine;
mod the_protected_defaults_match_names_a_manager_would_report;
mod the_purge_ratio_is_a_setting_and_zero_turns_it_off;
mod the_ratio_catches_the_small_machine_a_count_misses;

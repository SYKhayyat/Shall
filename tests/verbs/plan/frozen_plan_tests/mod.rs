pub use shall::app::sync::resolver::StateResolver;
pub use shall::app::sync::saved_plan::PlanRemoval;
pub use shall::core::lock_kind::{self, LockKind, LockSelection};
pub use shall::core::{GraphAction, PackageSpec};
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::plan::*;
pub use shall::verbs::prelude::*;
pub use shall::verbs::sync::{enforce_policy, print_vars_changed};

fn spec(backend: &str, name: &str, requires: &[&str]) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        backend: backend.into(),
        options: Default::default(),
        requires: requires.iter().map(|s| s.to_string()).collect(),
        present: true,
    }
}
mod a_frozen_plan_keeps_the_ordering_its_specs_declare;
mod a_frozen_removal_is_in_the_graph_and_in_the_tracker;
mod a_plan_with_no_requires_has_no_edges_to_split_the_batch;
mod a_requirement_outside_the_plan_adds_nothing;
mod the_engine_guard_can_see_a_frozen_plans_removals;

pub use shall::app::holds::*;
pub use shall::core::{PackageSpec, StateRegistry};
pub use std::collections::{BTreeSet, HashMap};

fn spec(backend: &str, name: &str, hold: bool) -> PackageSpec {
    let mut s = PackageSpec {
        name: name.into(),
        backend: backend.into(),
        ..Default::default()
    };
    if hold {
        s.options.set("hold", "true");
    }
    s.options.set("__source", "modules/tools.txt:3");
    s
}
fn desired(specs: Vec<PackageSpec>) -> HashMap<String, Vec<PackageSpec>> {
    let mut out: HashMap<String, Vec<PackageSpec>> = HashMap::new();
    for s in specs {
        out.entry(s.backend.clone()).or_default().push(s);
    }
    out
}
mod a_bare_ledger_entry_still_matches_any_backend;
mod a_declared_hold_counts_and_so_does_a_ledger_one;
mod a_manifest_that_would_not_resolve_is_not_a_manifest_with_no_holds;
mod an_absent_declaration_holds_nothing;
mod an_unresolvable_model_leaves_the_ledger_holds_working;
mod each_hold_is_released_by_the_command_that_made_it;

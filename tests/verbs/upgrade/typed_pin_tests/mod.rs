pub use shall::app::sync::resolver::StateResolver;
pub use shall::core::manager::{BackendCapabilities, BackendCore};
pub use shall::core::PackageSpec;
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::prelude::*;
pub use shall::verbs::setup::handle_canary;
pub use shall::verbs::sync::{enforce_policy, print_flight_plan};
pub use shall::verbs::upgrade::*;
use std::collections::HashMap;
use std::sync::Arc;

fn spec(name: &str, version: Option<&str>, present: bool) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if let Some(v) = version {
        options.set("version", v.to_string());
    }
    PackageSpec {
        name: name.into(),
        backend: "apt".into(),
        options,
        requires: vec![],
        present,
    }
}
fn desired(specs: Vec<PackageSpec>) -> HashMap<String, Vec<PackageSpec>> {
    HashMap::from([("apt".to_string(), specs)])
}
fn registry(runs_here: bool) -> BackendRegistry {
    struct Fake {
        here: bool,
    }
    #[async_trait::async_trait]
    impl BackendCore for Fake {
        fn name(&self) -> &str {
            "apt"
        }
        fn is_available(&self) -> bool {
            self.here
        }
        fn probes(&self) -> Vec<String> {
            Vec::new()
        }
        fn needs_root(&self) -> bool {
            false
        }
    }
    let mut reg = BackendRegistry::new();
    reg.register(Arc::new(
        BackendCapabilities::builder(Arc::new(Fake { here: runs_here })).build(),
    ));
    reg
}
mod only_typed_concrete_versions_on_present_managers_are_pins;
mod the_names_come_out_in_one_order;

pub use shall::app::backends::*;
pub use shall::backends::BackendRegistry;
pub use shall::core::manager::BackendCapabilities;
pub use shall::core::{Error, Result};
pub use shall::model::priority::Priority;
use std::sync::Arc;

struct Present(String);
impl shall::core::BackendCore for Present {
    fn name(&self) -> &str {
        &self.0
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}
fn registry_of(names: &[&str]) -> Arc<BackendRegistry> {
    let mut reg = BackendRegistry::new();
    for name in names {
        reg.register(Arc::new(
            BackendCapabilities::builder(Arc::new(Present(name.to_string()))).build(),
        ));
    }
    Arc::new(reg)
}
mod a_priority_that_did_not_resolve_is_refused_and_never_widened;
mod naming_a_backend_narrows_to_it_and_never_widens;
mod usable_is_the_priority_order_and_nothing_else;
mod usable_keeps_the_file_order_rather_than_the_registrys;

use async_trait::async_trait;
pub use shall::app::sync::pins::*;
pub use shall::backends::{capability, BackendRegistry};
pub use shall::core::manager::{BackendCapabilities, BackendCore, Installable};
pub use shall::core::PackageSpec;
use std::sync::Arc;

struct Fake {
    name: String,
    pins: bool,
    here: bool,
}
impl BackendCore for Fake {
    fn name(&self) -> &str {
        &self.name
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
#[async_trait]
impl Installable for Fake {
    async fn install(&self, _: &[PackageSpec], _: bool) -> shall::core::Result<()> {
        Ok(())
    }
    async fn remove(
        &self,
        _: &[String],
        _: bool,
        _: shall::app::sync::guard::Reaped,
    ) -> shall::core::Result<()> {
        Ok(())
    }
    fn pins_version(&self) -> bool {
        self.pins
    }
}
fn registry(backends: &[(&str, bool, bool)]) -> BackendRegistry {
    let mut reg = BackendRegistry::new();
    for (name, pins, here) in backends {
        let fake = Arc::new(Fake {
            name: (*name).to_string(),
            pins: *pins,
            here: *here,
        });
        reg.register(Arc::new(
            BackendCapabilities::builder(fake.clone())
                .with_installable(fake)
                .build(),
        ));
    }
    reg
}
fn spec(name: &str, backend: &str, version: Option<&str>) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if let Some(v) = version {
        options.set("version", v.to_string());
    }
    PackageSpec {
        name: name.into(),
        backend: backend.into(),
        options,
        requires: vec![],
        present: true,
    }
}
fn found(reg: &BackendRegistry, specs: &[PackageSpec]) -> Vec<String> {
    unmeetable(reg, specs.iter().map(|s| (s.backend.as_str(), s)))
        .into_iter()
        .map(|p| p.key)
        .collect()
}
mod a_manager_that_is_not_here_raises_nothing;
mod a_pin_is_unmeetable_only_where_the_manager_cannot_express_one;
mod an_absent_declaration_pins_nothing;
mod only_a_concrete_version_is_a_pin;
mod the_message_carries_the_ledger_s_reason;

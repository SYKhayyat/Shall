pub use shall::app::apply::extras::in_effect;
pub use shall::app::apply::extras::*;
pub use shall::app::sync::guard;
pub use shall::backends::service::{InitProviderFile, ServiceBackendCore, ServiceQueryable};
pub use shall::backends::BackendRegistry;
pub use shall::config::grammar::{Options, Origin, Statement};
pub use shall::core::extras_lock::ExtraKey;
pub use shall::core::{BackendCapabilities, CommandExecutor, Error, LockFile, Result};
use std::sync::Arc;
use tracing::warn;

fn registry_listing_nginx() -> BackendRegistry {
    #[cfg(windows)]
    let toml = r#"
[[init]]
name = "probe"
detect = "cmd"
start = [["cmd", "/C", "exit 0"]]
stop  = [["cmd", "/C", "exit 0"]]
list  = ["cmd", "/C", "echo SERVICE_NAME: nginx"]
list_pattern = 'SERVICE_NAME:\s+(\S+)'
"#;
    #[cfg(not(windows))]
    let toml = r#"
[[init]]
name = "probe"
detect = "sh"
start = [["sh", "-c", "exit 0"]]
stop  = [["sh", "-c", "exit 0"]]
list  = ["sh", "-c", "echo 'SERVICE_NAME: nginx'"]
list_pattern = 'SERVICE_NAME:\s+(\S+)'
"#;
    let file: InitProviderFile = toml::from_str(toml).expect("the probe row parses");
    let core = Arc::new(ServiceBackendCore::with_providers(
        CommandExecutor::new(false, false),
        file.init,
    ));
    let mut reg = BackendRegistry::new();
    reg.register(Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_queryable(Arc::new(ServiceQueryable { core }))
            .build(),
    ));
    reg
}
fn service(name: &str, key: &str, value: &str) -> (Statement, ExtraKey) {
    let mut opts = Options::default();
    opts.insert(key, value);
    (
        Statement::Service(name.to_string(), opts),
        ExtraKey::new(shall::config::grammar::ResourceKind::Service, name),
    )
}
mod a_directory_participates_in_convergence_for_type_mode_and_owner;
mod a_rendered_template_is_read_back_rendered_not_raw;
mod a_service_already_in_its_declared_state_is_not_placed_again;
mod each_kind_either_answers_or_says_why_it_cannot;
mod what_the_listing_cannot_answer_is_left_unverifiable;

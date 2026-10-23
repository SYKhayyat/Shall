pub use shall::app::sync::resolver::StateResolver;
pub use shall::core::lock_kind::{self, LockKind, LockSelection};
pub use shall::core::state::ManagedPackage;
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::plan::*;
pub use shall::verbs::prelude::*;
pub use shall::verbs::sync::{enforce_policy, print_vars_changed};

fn pkg(backend: &str, name: &str, unverified: bool) -> ManagedPackage {
    ManagedPackage {
        name: name.into(),
        backend: backend.into(),
        version: None,
        installed_at: 0,
        expires_at: None,
        options: if unverified {
            [("unverified".to_string(), "true".to_string())]
                .into_iter()
                .collect()
        } else {
            Default::default()
        },
        source: "test".into(),
        is_transient: false,
        session_id: None,
    }
}
mod the_heading_does_not_claim_shall_downloaded_it;
mod what_skipped_a_check_is_listed_whichever_backend_skipped_it;

pub use shall::backends::shared_database::*;
pub use shall::backends::BackendRegistry;
pub use shall::core::Package;
pub use std::collections::{HashMap, HashSet};

fn pkg(backend: &str, name: &str) -> shall::core::Package {
    shall::core::Package {
        backend: backend.into(),
        name: name.into(),
        version: None,
        properties: Default::default(),
    }
}
mod a_listing_from_a_client_alone_keeps_its_rows;
mod a_package_the_owner_cannot_reinstall_is_named_under_the_client_that_can;
mod a_shared_database_yields_one_holder_and_never_none;
mod an_empty_set_reports_nothing_foreign;
mod one_installed_package_is_one_row_however_many_clients_answered;
mod only_the_aur_helpers_read_another_backends_database;
mod the_holder_collapse_follows_the_same_rule_as_the_row_collapse;
mod the_manager_level_collapse_is_unchanged_by_any_of_this;
mod the_owner_keeps_a_foreign_package_when_no_client_answered;

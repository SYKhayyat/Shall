use serde::Deserialize;
pub use shall::core::adapter::{self, AdapterRow};
pub use shall::model::secret::*;
use std::path::Path;
use std::time::Duration;

fn provider(name: &str, decrypt: &[&str], stdout_only: bool) -> SecretProvider {
    SecretProvider {
        name: name.into(),
        os: None,
        decrypt: decrypt.iter().map(|s| s.to_string()).collect(),
        stdout_only,
    }
}
mod a_plugin_identity_is_recognised_by_name;
mod a_provider_can_be_restricted_to_one_os;
mod a_provider_command_fills_the_reference_and_identity;
mod a_provider_with_no_name_or_command_is_refused;
mod a_second_provider_claiming_a_name_is_refused;
mod a_secret_provider_must_promise_stdout_only;
mod a_software_identity_has_no_plugin;
mod providers_drops_the_unusable_and_keeps_the_rest;
mod the_provider_schema_parses;
mod the_timeout_message_names_the_token_and_the_identity;
mod the_watch_skip_message_says_why_and_when_it_will_apply;

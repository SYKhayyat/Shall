pub use shall::config::parser::split_removal_target;
pub use shall::config::parser::*;
pub use shall::core::{Error, Result};
pub use shall::model::vars::Value;

fn known(b: &str) -> bool {
    matches!(b, "apt" | "uv" | "npm" | "web" | "cargo")
}
mod backend_prefix_scopes_and_strips_to_bare_name;
mod bare_name_has_no_backend;
mod options_are_stripped_from_the_name;
mod unknown_prefix_is_not_treated_as_a_backend;
mod web_url_name_keeps_its_scheme_colon;

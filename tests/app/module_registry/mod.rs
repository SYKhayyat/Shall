pub use shall::app::module_registry::*;
pub use shall::core::{Error, Result};
mod entry_count_and_html_guard;
mod github_user_repo_defaults_to_repo_module;
mod github_with_explicit_path_and_ref;
mod raw_https_url_is_used_verbatim;
mod rejects_bad_sources;

pub use shall::backends::node_registry::http_timeout;
pub use shall::backends::pip_search::*;
pub use shall::core::{Error, Package, Result};
mod parses_pypi_info;
mod pypi_falls_back_to_query_name;
mod the_query_is_path_encoded_not_pasted_into_the_url;
mod the_result_is_tagged_with_the_backend_that_asked;

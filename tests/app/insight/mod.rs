pub use serde_json::{json, Value};
pub use shall::app::insight::*;
pub use shall::backends::BackendRegistry;
pub use shall::config::Config;
pub use shall::core::{Error, LockFile, Output, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

fn p(backend: &str, name: &str, version: Option<&str>) -> ResolvedPkg {
    ResolvedPkg {
        backend: backend.into(),
        name: name.into(),
        version: version.map(String::from),
    }
}
fn opts(pairs: &[(&str, &str)]) -> shall::config::grammar::Options {
    let mut o = shall::config::grammar::Options::default();
    for (k, v) in pairs {
        for part in v.split(';') {
            o.insert(*k, part);
        }
    }
    o
}
mod a_backend_that_installs_one_artifact_gets_no_formats_line;
mod a_declaration_reads_as_a_place_you_can_open;
mod a_declaration_with_no_scope_still_names_its_file;
mod a_lapsed_declaration_says_it_stopped_counting;
mod a_package_a_pattern_matched_names_the_pattern;
mod an_absent_tag_is_the_built_in_default_and_prints_this_machines_order;
mod cyclonedx_has_expected_shape;
mod extracts_fixed_version_and_summary;
mod parses_querybatch_ids_aligned;
mod purl_and_ecosystem_maps;
mod querybatch_skips_uncovered_and_floating;
mod the_line_says_so_when_the_declaration_carried_formats;
mod the_priority_file_says_so_when_the_backend_body_won;
mod why_names_each_writer_by_the_string_that_writer_stores;

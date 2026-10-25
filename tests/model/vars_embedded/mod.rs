pub use rhai::{Dynamic, Scope};
pub use shall::config::grammar::{GrammarError, Origin, Result};
pub use shall::config::parser::HostFacts;
pub use shall::model::vars::{Value, VarOrigins, Vars};
pub use shall::model::vars_embedded::*;
use std::path::Path;
pub use std::sync::atomic::{AtomicUsize, Ordering};

fn facts() -> HostFacts {
    HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: None,
        user: None,
        vars: Default::default(),
    }
}
fn run(body: &str) -> Result<Vars> {
    static N: AtomicUsize = AtomicUsize::new(0);
    // `env::temp_dir()`, not TMP-or-TMPDIR-or-".": neither variable is set in a plain
    // Linux shell, so the fallback was the current directory — which is the repo, and
    // every `cargo test` left a pile of `shall-embedded-*.shall` in it.
    let path = std::env::temp_dir().join(format!(
        "shall-embedded-{}-{}.shall",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, body).unwrap();
    resolve(&path, &facts())
}
mod a_failing_shell_command_throws;
mod a_file_can_be_read_and_probed;
mod a_map_of_the_four_types_resolves;
mod a_map_valued_variable_is_refused;
mod a_read_of_a_missing_file_throws_rather_than_resolving_to_nothing;
mod a_script_that_does_not_end_in_a_map_is_an_error;
mod an_http_get_to_a_dead_address_throws;
mod json_can_be_parsed_and_navigated_for_a_scalar;
mod the_clock_is_available;
mod the_environment_is_readable_and_is_w7s_escape_hatch;
mod the_script_can_compute;
mod the_script_can_read_the_detected_facts;
mod the_shell_runs_and_a_check_variant_does_not_throw;

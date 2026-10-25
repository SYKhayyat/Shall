pub use shall::config::grammar::error::{GrammarError, Origin, Result};
pub use shall::config::grammar::options::Options;
pub use shall::config::grammar::{gated, Vocabulary};
pub use shall::config::parser::HostFacts;
pub use shall::model::priority::*;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

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
fn parse(body: &str) -> Result<Priority> {
    Priority::parse(&PathBuf::from("priority"), body, &facts())
}
mod a_backend_line_may_carry_its_defaults;
mod a_backend_named_twice_keeps_its_first_position;
mod a_bare_name_still_carries_no_options;
mod a_body_inside_a_when_block_applies_only_there;
mod a_body_line_without_an_equals_is_an_error;
mod a_body_listing_a_backend_also_enables_it;
mod a_line_that_is_not_a_backend_name_is_an_error;
mod a_plain_list_keeps_its_order;
mod a_stray_brace_is_an_error;
mod a_system_manager_outranks_a_language_manager;
mod a_variable_block_chooses_a_backend;
mod a_when_block_gates_the_backends_inside_it;
mod an_option_the_backend_cannot_read_is_an_error_here_too;
mod an_unclosed_body_is_an_error_naming_the_backend;
mod an_unclosed_when_is_an_error;
mod an_unknown_backend_does_not_outrank_your_distro;
mod an_unknown_format_in_a_body_is_an_error_naming_the_real_ones;
mod comments_and_blanks_are_skipped;
mod not_listed_means_shall_does_not_use_it;
mod only_what_this_machine_has_is_listed;
mod pip_is_last_because_it_can_break_your_system_python;
mod service_and_link_are_not_listed_but_artifact_backends_are;
mod the_bootstrap_pass_names_every_backend_and_evaluates_nothing;
mod the_refusal_says_how_to_fix_it;
mod the_starter_file_carries_its_reason;
mod the_when_arm_that_matched_beats_the_plain_line_below_it;

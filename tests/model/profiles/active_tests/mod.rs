pub use shall::config::grammar::{
    gated, parse_document, BackendNames, Gates, GrammarError, Origin, Reference, Result, Statement,
    Vocabulary,
};
pub use shall::config::parser::HostFacts;
pub use shall::model::cycle::{self, Hop, Visit};
pub use shall::model::layout::Layout;
pub use shall::model::profiles::*;
use std::path::PathBuf;

fn facts(host: &str) -> HostFacts {
    HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: host.into(),
        family: "debian".into(),
        home: None,
        user: None,
        vars: Default::default(),
    }
}
fn read(body: &str, host: &str) -> Result<Vec<ActiveEntry>> {
    read_active(&PathBuf::from("active"), body, &facts(host))
}
fn on(body: &str, host: &str) -> Vec<String> {
    read(body, host)
        .unwrap()
        .into_iter()
        .filter(|e| e.on)
        .map(|e| e.name)
        .collect()
}
const II6_EXAMPLE: &str = "Work\nGaming\n\nwhen host == laptop {\n  Travel\n}\n";
fn deactivate(body: &str, host: &str, names: &[&str]) -> ActiveEdit {
    let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    remove_from_active(&PathBuf::from("active"), body, &names, &facts(host)).unwrap()
}
mod a_block_for_another_host_is_never_touched;
mod a_block_with_another_name_left_in_it_survives;
mod a_gated_name_is_in_the_file_and_says_which_block_holds_it;
mod a_lowercase_name_is_not_a_profile;
mod a_repeat_is_ignored;
mod a_when_block_in_active_can_test_a_variable;
mod activate_can_name_every_block_it_overwrites;
mod active_holds_names_never_expressions;
mod an_unclosed_or_stray_block_is_an_error;
mod comments_and_blanks_are_skipped;
mod deactivate_reaches_into_a_block_that_applies_here;
mod deactivate_takes_a_top_level_name_out;
mod deactivating_a_name_that_is_not_there_changes_nothing_and_says_so;
mod the_example_in_the_spec_parses;
mod when_gates_active_like_every_other_file;

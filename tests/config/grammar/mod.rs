pub use error::{GrammarError, Origin, Result};
pub use gated::{GatedLine, Vocabulary};
pub use options::Options;
pub use shall::config::grammar::*;
pub use shall::config::parser::{eval_when, HostFacts};
mod error;
mod exec;
mod options;
mod statement;
use std::path::Path;
use std::path::PathBuf;

fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo" | "snap")
}
fn doc(body: &str) -> Result<Document> {
    parse_document(&PathBuf::from("modules/dev.txt"), body, &known)
}
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
fn stmts(body: &str) -> Vec<Statement> {
    doc(body)
        .unwrap()
        .statements_for(&facts())
        .unwrap()
        .into_iter()
        .map(|(s, _)| s)
        .collect()
}
mod a_block_body_is_held_to_the_same_rules_as_the_short_form;
mod a_declaration_block_body_is_options;
mod a_declaration_block_keeps_commas_and_equals_in_a_value;
mod a_hash_inside_a_value_is_data_and_a_hash_after_a_space_is_a_comment;
mod a_key_given_twice_in_a_block_makes_a_list;
mod a_module_block_body_is_lines;
mod a_module_block_must_be_lowercase;
mod a_name_that_declares_something_other_than_a_package_says_so;
mod a_retired_option_cannot_be_reached_through_a_block;
mod a_stray_closing_brace_is_an_error;
mod a_trailing_comment_is_stripped_from_a_statement;
mod an_inner_user_overrides_the_block_user;
mod an_unclosed_block_is_an_error;
mod an_unknown_when_key_is_an_error_that_lists_the_real_ones;
mod an_unrecognised_line_is_an_error_not_a_package_name;
mod blank_lines_and_whole_line_comments_are_skipped;
mod every_statement_form_parses;
mod individual_user_and_owner_options_are_validated;
mod statements_carry_where_they_came_from;
mod use_user_block_adds_user_option_to_dir;
mod use_user_block_adds_user_option_to_link;
mod use_user_block_adds_user_option_to_service;
mod use_user_block_rejects_empty_username;
mod use_user_block_rejects_nested_blocks;
mod use_user_block_rejects_unsupported_statements;
mod use_user_block_supports_all_six_scopable_statements;
mod when_gates_the_lines_inside_it;
mod when_nests_inside_a_module;
mod when_supports_inequality_and_membership;

use super::*;

pub use shall::config::grammar::statement::*;
fn o() -> Origin {
    Origin::new("modules/dev.txt", 7)
}
fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo" | "snap" | "npm")
}
fn p(line: &str) -> Result<Statement> {
    parse(&o(), line, &known)
}
struct WithGroup;
impl BackendNames for WithGroup {
    fn is_backend(&self, name: &str) -> bool {
        known(name)
    }
    fn expand_group(&self, name: &str) -> Option<Vec<String>> {
        (name == "web").then(|| vec!["cargo".to_string(), "npm".to_string()])
    }
}
fn cands(line: &str) -> (Option<String>, Candidates) {
    let Statement::Package(d) = p(line).unwrap() else {
        panic!("`{}` did not parse as a package", line)
    };
    (d.backend, d.candidates)
}

mod a_backend_named_twice_in_a_chain_is_refused;
mod a_backend_not_in_priority_says_so;
mod a_bare_name_has_no_backend;
mod a_bare_requires_is_an_error;
mod a_chain_can_end_in_the_whole_priority_list;
mod a_chain_only_accepts_what_it_lists;
mod a_group_composes_with_a_backend_in_the_chain;
mod a_group_prefix_expands_to_its_chain;
mod a_health_check_that_is_not_a_check_is_refused;
mod a_link_naming_two_modes_is_refused_naming_both;
mod a_link_template_that_is_neither_on_nor_off_is_refused;
mod a_lone_backend_pins_and_a_chain_does_not;
mod a_pattern_cannot_span_a_chain;
mod a_prefix_whose_payload_looks_like_set_math_is_still_that_statement;
mod a_profile_cannot_take_arguments;
mod a_qualified_requires_is_accepted;
mod a_quoted_name_carries_the_spaces_a_manager_reports;
mod a_regex_must_say_which_backend;
mod a_regex_selects_by_pattern;
mod a_relative_expiry_is_an_error;
mod a_repo_without_a_backend_is_refused;
mod a_setting_names_a_schema_a_key_and_a_value;
mod a_setting_takes_one_value_not_two;
mod a_setting_with_no_value_declares_nothing;
mod a_setting_without_a_slash_is_not_schema_key;
mod a_typo_on_a_setting_is_refused_like_any_other_extra;
mod a_typo_on_an_extra_is_refused_by_name;
mod absent_declares_a_package_must_not_exist;
mod absent_must_name_a_backend;
mod an_absolute_expiry_is_accepted;
mod an_empty_slot_in_a_chain_is_refused;
mod an_explicit_backend_is_read;
mod an_unclosed_use_paren_is_an_error;
mod an_unknown_backend_inside_a_chain_is_still_unknown;
mod an_unknown_backend_prefix_is_an_error_not_a_package_name;
mod both_shapes_of_health_check_parse;
mod every_error_names_the_file_and_line;
mod every_prefix_advertises_a_line_this_grammar_accepts;
mod hold_and_version_together_are_a_contradiction;
mod keyword_roles_and_spellings_agree;
mod list_is_how_a_bare_name_is_spelled_out;
mod no_reserved_word_is_listed_twice;
mod nothing_may_follow_list_in_a_chain;
mod only_prefixes_build_statements;
mod param_names_must_be_identifiers;
mod param_parses_with_and_without_a_default;
mod repo_and_the_package_needing_it_are_both_statements;
mod shim_carries_its_source;
mod the_documented_keys_on_an_extra_are_accepted;
mod the_order_asked_is_the_order_written_then_priority;
mod the_reserved_words_are_the_ones_the_prefix_grammar_spends;
mod the_resource_prefixes_are_one_list;
mod use_argument_values_may_contain_slashes;
mod use_never_takes_a_path_or_a_url;
mod use_parses_module_arguments;
mod use_takes_a_module_by_lowercase_name;
mod use_takes_a_profile_by_capitalized_name;

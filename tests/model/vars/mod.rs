pub use shall::config::grammar::{GrammarError, Origin, Result};
pub use shall::model::vars::*;
pub use std::collections::{BTreeMap, HashSet};

fn origin(line: usize) -> Origin {
    Origin::new("vars", line)
}
fn top(name: &str, value: &str, line: usize) -> Definition {
    Definition {
        name: name.into(),
        value: value.into(),
        origin: origin(line),
        conditional: false,
    }
}
fn when(name: &str, value: &str, line: usize) -> Definition {
    Definition {
        name: name.into(),
        value: value.into(),
        origin: origin(line),
        conditional: true,
    }
}
fn str_val(s: &str) -> Value {
    Value::Str(s.into())
}
fn expand_facts() -> shall::config::parser::HostFacts {
    shall::config::parser::HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: Some("/home/u".into()),
        user: Some("u".into()),
        vars: Vars::new(),
    }
}
mod a_bare_true_or_false_is_a_boolean_not_a_string;
mod a_bracketed_value_is_a_list_of_typed_elements;
mod a_cycle_names_the_whole_loop;
mod a_default_survives_when_nothing_overrides_it;
mod a_doubled_dollar_is_a_literal_one;
mod a_list_cannot_be_interpolated_into_a_string_value;
mod a_matching_block_overrides_the_default;
mod a_number_displays_without_a_trailing_zero;
mod a_plain_number_is_a_number_and_a_version_is_not;
mod a_reference_ends_at_a_non_name_character_without_braces;
mod a_shell_positional_is_not_a_variable_reference;
mod a_value_may_be_built_from_another_variable;
mod a_value_that_is_exactly_one_reference_inherits_its_type;
mod a_variable_defined_only_inside_a_block_is_an_error;
mod a_variable_that_references_itself_is_a_cycle;
mod an_override_is_visible_to_everything_derived_from_it;
mod braces_are_what_let_a_reference_touch_a_name_character;
mod derived_values_resolve_in_dependency_order_not_file_order;
mod diff_reports_changed_added_and_gone_but_not_case_only;
mod expand_leaves_a_value_with_no_references_alone;
mod expand_names_the_fact_vocabulary_when_nothing_answers;
mod expand_of_an_undetectable_fact_is_an_error_not_an_empty_string;
mod expand_prefers_a_variable_you_decided_over_a_fact_the_machine_reported;
mod expand_reads_a_detected_fact_when_no_variable_names_it;
mod expand_refuses_a_list_rather_than_joining_it;
mod expand_refuses_an_unknown_name_rather_than_leaving_it_literal;
mod expand_stringifies_a_number_or_boolean;
mod expand_substitutes_into_a_value_written_outside_vars;
mod one_name_cannot_have_two_defaults;
mod ordering_is_defined_only_between_numbers;
mod parse_number_rejects_infinity_and_multiple_dots;
mod quotes_force_a_string_even_when_it_looks_like_another_type;
mod referenced_names_finds_every_reference_and_skips_escapes;
mod referring_to_a_name_that_does_not_exist_is_an_error;
mod the_origin_is_the_winning_definition_not_the_default;
mod two_matching_blocks_that_agree_are_redundant_not_wrong;
mod two_matching_blocks_that_disagree_name_both_lines;
mod value_equality_has_no_cross_type_coercion;

pub use shall::config::parser::*;
pub use shall::core::{Error, Result};
pub use shall::model::vars::Value;

fn facts() -> HostFacts {
    HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: Some("/home/u".into()),
        user: Some("u".into()),
        vars: Default::default(),
    }
}
fn with_role(role: &str) -> HostFacts {
    let mut vars = shall::model::vars::Vars::new();
    vars.insert("role".to_string(), Value::Str(role.into()));
    facts().with_vars(vars)
}
mod a_bare_variable_is_not_a_condition;
mod a_derivative_resolves_to_the_family_that_decides_the_artifact;
mod a_fact_the_machine_cannot_answer_is_an_error_naming_the_fact;
mod a_family_that_is_not_debian_makes_the_comparison_false;
mod a_variable_can_never_shadow_a_detected_fact;
mod a_variable_is_reached_with_the_sigil;
mod an_os_release_with_no_id_at_all_answers_nothing;
mod an_undefined_variable_is_an_error_not_a_silent_false;
mod an_unknown_distribution_reports_its_own_name_rather_than_nothing;
mod eval_equality_and_inequality;
mod eval_membership;
mod eval_rejects_unknown_key_and_bad_syntax;
mod family_answers_the_distribution_and_os_answers_the_kernel;
mod family_is_never_empty;
mod home_and_user_are_facts_the_machine_reports;
mod ordering_a_string_is_refused_not_answered_wrongly;
mod quotes_around_a_value_are_not_part_of_it;
mod the_detected_home_agrees_with_the_tilde_the_link_backend_reads;
mod the_rhel_derivatives_share_one_family;
mod typed_comparisons_have_no_cross_type_coercion;
mod ubuntu_is_debian_even_without_an_id_like;

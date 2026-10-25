pub use shall::config::grammar::error::{GrammarError, Origin, Result};
pub use shall::config::grammar::exec::*;
pub use shall::config::grammar::options::Options;
pub use shall::config::grammar::statement::{parse, validate, ResourceKind, Statement};
pub use shall::config::grammar::statement::{validate_extra_options, OptionKind};
pub use shall::model::exec::Verb as ExecVerb;

fn o() -> Origin {
    Origin::new("modules/dev.txt", 7)
}
fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo")
}
fn p(line: &str) -> Result<Statement> {
    parse(&o(), line, &known)
}
fn pv(line: &str) -> Result<Statement> {
    let s = p(line)?;
    validate(&o(), &s)?;
    Ok(s)
}
mod a_generate_names_a_command;
mod a_generate_takes_no_options;
mod a_generate_with_no_command_is_an_error;
mod a_windows_path_is_a_script_not_an_expression;
mod a_zero_or_garbage_ceiling_is_refused;
mod an_exec_is_not_an_extra_with_a_teardown_key;
mod an_exec_names_a_script;
mod an_exec_takes_a_runs_ceiling;
mod an_exec_that_names_nothing_is_refused;
mod an_unknown_exec_option_is_refused_and_names_the_real_one;
mod every_resource_kind_round_trips_through_its_keyword;
mod every_statement_with_a_keyword_reports_a_listed_kind;
mod on_names_one_of_the_three_verbs_and_nothing_else;

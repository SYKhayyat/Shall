pub use shall::config::grammar::{
    parse_document, BackendNames, Document, Gates, GrammarError, Origin, Reference, Result,
    Statement,
};
pub use shall::config::parser::HostFacts;
pub use shall::model::cycle::{self, Hop, Visit};
pub use shall::model::layout::{Layout, ModuleName};
pub use shall::model::modules::*;
use std::collections::HashMap;
use tempfile::TempDir;

fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo")
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
struct Fixture {
    _tmp: TempDir,
    layout: Layout,
}
fn fixture(files: &[(&str, &str)]) -> Fixture {
    let tmp = TempDir::new().unwrap();
    let layout = Layout::new(tmp.path().join("cfg"), tmp.path().join("data"));
    std::fs::create_dir_all(layout.modules_dir()).unwrap();
    for (name, body) in files {
        std::fs::write(layout.modules_dir().join(name), body).unwrap();
    }
    Fixture { _tmp: tmp, layout }
}
fn expand_module(f: &Fixture, name: &str) -> Result<Vec<Statement>> {
    let mut loader = ModuleLoader::new(&f.layout, &known);
    let out = expand(
        &mut loader,
        name,
        &Origin::argument(),
        &facts(),
        &mut Vec::new(),
        &Vec::new(),
    )?;
    Ok(out.into_iter().map(|(s, _, _)| s).collect())
}
fn expand_with_args(f: &Fixture, name: &str, args: &[(&str, &str)]) -> Result<Vec<Statement>> {
    let args: Vec<(String, String)> = args
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let mut loader = ModuleLoader::new(&f.layout, &known);
    let out = expand_args(
        &mut loader,
        name,
        &Origin::argument(),
        &facts(),
        &mut Vec::new(),
        &Vec::new(),
        &args,
    )?;
    Ok(out.into_iter().map(|(s, _, _)| s).collect())
}
fn link_target(stmt: &Statement) -> Option<&str> {
    match stmt {
        Statement::Link(_, opts) => opts.one("target"),
        _ => None,
    }
}
mod a_diamond_is_not_a_loop;
mod a_missing_module_names_what_is_available;
mod a_module_can_never_reference_a_profile;
mod a_module_can_use_another_module;
mod a_module_is_a_list_of_lines;
mod a_module_that_uses_itself_is_an_error_not_a_hang;
mod a_nested_use_receives_substituted_arguments;
mod a_param_default_is_used_when_no_argument_is_passed;
mod a_param_gates_a_when_block;
mod a_param_is_substituted_from_a_use_argument;
mod a_plain_use_of_a_parameterless_module_is_unchanged;
mod a_required_param_with_no_argument_is_a_loud_error;
mod a_wrong_case_reference_teaches_the_rule;
mod an_argument_that_names_no_parameter_is_an_error;
mod an_unknown_dollar_reference_is_left_for_the_global_vars_pass;
mod only_txt_files_are_modules_so_a_readme_costs_nothing;
mod only_what_is_reached_is_parsed;
mod the_filename_is_the_module_name_lowercased;
mod when_gates_lines_inside_a_module;

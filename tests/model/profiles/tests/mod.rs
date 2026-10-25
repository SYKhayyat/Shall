pub use shall::config::grammar::{
    gated, parse_document, BackendNames, Gates, GrammarError, Origin, Reference, Result, Statement,
    Vocabulary,
};
pub use shall::config::parser::HostFacts;
pub use shall::model::cycle::{self, Hop, Visit};
pub use shall::model::layout::Layout;
pub use shall::model::profiles::*;
use std::path::PathBuf;
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
fn facts_with(pairs: &[(&str, &str)]) -> HostFacts {
    let mut f = facts();
    for (k, v) in pairs {
        f.vars
            .insert(k.to_string(), shall::model::vars::Value::parse_literal(v));
    }
    f
}
struct Fixture {
    _tmp: TempDir,
    layout: Layout,
}
fn fixture(profiles: &[(&str, &str)], modules: &[(&str, &str)]) -> Fixture {
    let tmp = TempDir::new().unwrap();
    let layout = Layout::new(tmp.path().join("cfg"), tmp.path().join("data"));
    std::fs::create_dir_all(layout.profiles_dir()).unwrap();
    std::fs::create_dir_all(layout.modules_dir()).unwrap();
    for (n, b) in profiles {
        std::fs::write(layout.profiles_dir().join(n), b).unwrap();
    }
    for (n, b) in modules {
        std::fs::write(layout.modules_dir().join(n), b).unwrap();
    }
    Fixture { _tmp: tmp, layout }
}
fn resolve(f: &Fixture, name: &str) -> Result<Resolved> {
    ProfileLoader::new(&f.layout, &known).resolve(
        name,
        &Origin::argument(),
        &facts(),
        &mut Vec::new(),
        &Vec::new(),
    )
}
fn module_names(r: &Resolved) -> Vec<&str> {
    r.modules.iter().map(|m| m.name.as_str()).collect()
}
mod a_message_about_a_variable_block_says_what_the_variable_is;
mod a_message_names_an_undefined_variable_rather_than_going_quiet;
mod a_missing_profile_that_matches_a_module_teaches_the_rule;
mod a_profile_chooses_modules;
mod a_profile_may_hold_package_lines_directly;
mod a_profile_may_reference_a_profile;
mod a_profile_that_uses_itself_is_an_error_not_a_hang;
mod a_variable_block_in_active_is_read_against_this_runs_variables;
mod active_ignores_a_repeat;
mod active_is_a_plain_list_of_profile_names;
mod active_refuses_a_module_name;
mod active_refuses_anything_that_is_not_a_name;

pub use chrono::{DateTime, Utc};
pub use shall::config::grammar::{
    statement, BackendNames, Candidates, Gate, Gates, GrammarError, Options, Origin, PackageDecl,
    Phase, Result, Selector, Statement,
};
pub use shall::config::parser::HostFacts;
pub use shall::core::LockFile;
pub use shall::core::PackageSpec;
pub use shall::model::conflict::{reconcile, Declared};
pub use shall::model::cycle::{self, Hop};
pub use shall::model::dated::dating_of;
pub use shall::model::dated::parse_absolute;
pub use shall::model::layout::Layout;
pub use shall::model::modules::{expand, expand_args, ModuleLoader};
pub use shall::model::priority::Priority;
pub use shall::model::profiles::{read_active, ProfileLoader, SetOp};
pub use shall::model::resolve::*;
pub use std::collections::{BTreeMap, BTreeSet, HashMap};
pub use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo" | "snap")
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
struct Fx {
    _tmp: TempDir,
    layout: Layout,
    priority: Priority,
}
fn fx(active: &str, profiles: &[(&str, &str)], modules: &[(&str, &str)]) -> Fx {
    let tmp = TempDir::new().unwrap();
    let layout = Layout::new(tmp.path().join("cfg"), tmp.path().join("data"));
    std::fs::create_dir_all(layout.profiles_dir()).unwrap();
    std::fs::create_dir_all(layout.modules_dir()).unwrap();
    std::fs::write(layout.active_file(), active).unwrap();
    for (n, b) in profiles {
        std::fs::write(layout.profiles_dir().join(n), b).unwrap();
    }
    for (n, b) in modules {
        std::fs::write(layout.modules_dir().join(n), b).unwrap();
    }
    Fx {
        _tmp: tmp,
        layout,
        priority: Priority::from_backends(vec!["apt".into(), "cargo".into()]),
    }
}
fn resolve(f: &Fx) -> Result<DesiredState> {
    Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
}
fn names(d: &DesiredState, backend: &str) -> Vec<String> {
    let mut v: Vec<String> = d
        .present()
        .filter(|p| p.backend == backend)
        .map(|p| p.name.clone())
        .collect();
    v.sort();
    v
}
fn load_vars(f: &Fx) -> Result<shall::model::vars::Vars> {
    Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .load_vars()
}
fn with_vars(f: &Fx, body: &str) {
    std::fs::write(f.layout.vars_file(), body).unwrap();
}
fn resolve_probed(f: &Fx, answers: &[(&str, &str)]) -> Result<DesiredState> {
    let table: HashMap<String, BareAnswer> = answers
        .iter()
        .map(|(n, b)| (n.to_string(), BareAnswer::to(b)))
        .collect();
    let r = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .with_bare(table);
    let stmts = r.statements()?;
    r.collect(stmts)
}
mod a_bare_name_and_an_explicit_one_are_one_package_once_probed;
mod a_bare_name_is_left_for_the_prober;
mod a_bare_subtraction_takes_the_package_out_whatever_backend_holds_it;
mod a_block_that_does_not_match_leaves_the_default;
mod a_condition_that_tests_no_variable_is_not_recorded;
mod a_config_with_no_extras_has_no_dependents;
mod a_gated_line_can_use_a_variable;
mod a_generate_line_is_refused_when_generators_are_off;
mod a_lapsed_line_stops_counting_but_is_reported_with_its_file_and_line;
mod a_matching_block_overrides_the_default;
mod a_module_cannot_do_set_math;
mod a_package_carries_every_variable_condition_that_admitted_it;
mod a_package_line_in_the_vars_file_is_refused;
mod a_package_surviving_set_math_still_knows_its_module;
mod a_probed_bare_name_can_contradict_an_explicit_one;
mod a_profile_can_hold_a_package_directly;
mod a_profile_can_subtract_one_package;
mod a_profile_cannot_use_absent;
mod a_profile_whose_set_math_refers_to_itself_is_an_error_not_an_overflow;
mod a_regex_keeps_its_pattern_for_the_prober;
mod a_repo_carries_its_backend;
mod a_repo_for_a_backend_not_in_priority_is_refused;
mod a_schedule_in_the_schedules_file_is_read_and_is_not_a_dependent;
mod a_schedule_outside_the_schedules_file_is_refused;
mod a_schedules_run_command_keeps_its_dollars_for_the_shell;
mod a_set_expression_unions_and_intersects;
mod a_variable_defined_only_in_a_block_is_an_error_even_where_the_block_misses;
mod a_variable_expands_inside_a_declaration_value;
mod a_variable_expands_inside_an_option_value;
mod a_variable_line_outside_the_vars_file_is_refused;
mod absent_lands_in_its_own_bucket;
mod active_profiles_choose_which_modules_are_read;
mod an_explicit_backend_not_in_priority_is_refused;
mod an_explicit_subtraction_leaves_the_other_backends_line_alone;
mod an_unapproved_executing_vars_provider_is_refused_until_the_ledger_approves_it;
mod an_unknown_variable_is_an_error_even_with_no_vars_file;
mod an_unreached_broken_module_is_never_parsed;
mod asking_what_a_profile_would_give_you_does_not_change_what_is_active;
mod bare_provenance_survives_into_the_install_spec;
mod check_reaches_a_module_no_profile_reaches;
mod dependents_are_the_after_package_extras_only;
mod every_spec_records_where_it_came_from;
mod exclude_subtracts_a_whole_modules_packages;
mod extras_are_carried_in_declaration_order;
mod include_says_to_write_use_instead;
mod intersect_keeps_only_what_is_in_both;
mod link_content_substitutes_the_machines_home_at_resolve_time;
mod no_vars_file_is_no_variables_and_not_an_error;
mod nothing_is_active_unless_a_profile_names_it;
mod qualified_provenance_does_not_look_bare_to_apply;
mod set_math_over_other_profiles_still_resolves;
mod subtraction_wins_however_you_order_the_lines;
mod the_probers_answer_becomes_the_backend;
mod the_resolved_state_carries_its_variables_for_the_plan_to_freeze;
mod two_modules_declaring_one_package_differently_is_an_error_naming_both;
mod two_profiles_whose_set_math_name_each_other_are_an_error_not_an_overflow;

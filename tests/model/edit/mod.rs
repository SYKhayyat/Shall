pub use shall::config::grammar::{
    statement, BackendNames, GrammarError, Origin, Result, Statement,
};
pub use shall::config::parser::HostFacts;
pub use shall::model::edit::*;
pub use shall::model::layout::{Layout, ModuleName};
pub use shall::model::modules::ModuleLoader;
pub use shall::model::profiles::{parse_active, ProfileLoader};
use std::path::PathBuf;
use tempfile::TempDir;

fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo" | "npm")
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
}
fn fx(files: &[(&str, &str)]) -> Fx {
    let tmp = TempDir::new().unwrap();
    let layout = Layout::new(tmp.path().join("cfg"), tmp.path().join("data"));
    std::fs::create_dir_all(layout.modules_dir()).unwrap();
    std::fs::create_dir_all(layout.profiles_dir()).unwrap();
    for (p, b) in files {
        let full = layout.config_root().join(p);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, b).unwrap();
    }
    Fx { _tmp: tmp, layout }
}
fn editor(f: &Fx) -> Editor<'_> {
    Editor::new(&f.layout, &known, facts(), Writes::ToDisk)
}
fn read(f: &Fx, p: &str) -> String {
    std::fs::read_to_string(f.layout.config_root().join(p)).unwrap_or_default()
}
mod a_bare_target_removes_the_package_under_any_backend;
mod a_line_the_grammar_cannot_read_is_refused_before_the_file_is_touched;
mod a_module_already_reached_is_not_wired_again;
mod a_new_landing_module_explains_itself;
mod a_service_line_is_declared_and_undeclared_by_name;
mod an_explicit_target_removes_only_that_backends_line;
mod installing_a_pinned_version_replaces_the_unpinned_line;
mod installing_with_nothing_active_refuses_rather_than_writing_a_line_nobody_reads;
mod installing_with_several_profiles_active_asks_which_one_rather_than_guessing;
mod installing_writes_a_line_to_the_landing_module;
mod into_a_named_profile_settles_the_question;
mod re_enabling_a_service_replaces_its_line_rather_than_doubling_it;
mod removing_a_package_named_after_a_backend_does_not_delete_that_backend;
mod removing_keeps_comments_and_blank_lines;
mod rewrite_backend_prefix_changes_only_the_prefix;
mod target_case_decides_module_or_profile;
mod teleport_moves_the_line_in_place_and_leaves_no_second_copy;
mod teleport_of_an_undeclared_package_changes_nothing;
mod the_first_write_wires_the_module_into_the_active_profile_and_says_so;
mod uninstall_reaches_every_module_the_active_profiles_hold;

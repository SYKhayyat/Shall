pub use shall::app::apply::dotfiles::*;
pub use shall::config::grammar::{Options, Origin, Statement};
pub use shall::core::{Error, PackageSpec, Result};
use tracing::info;

fn link(name: &str, target: &str) -> (Statement, Origin) {
    let mut opts = Options::default();
    opts.insert("target".to_string(), target);
    (
        Statement::Link(name.to_string(), opts),
        Origin::new("modules/files.txt", 1),
    )
}
fn state_with(extras: Vec<(Statement, Origin)>) -> shall::model::DesiredState {
    shall::model::DesiredState {
        extras,
        ..Default::default()
    }
}
mod a_trees_files_outside_home_are_asked_about_too;
mod only_a_new_link_outside_home_is_asked_about;
mod outside_home_uses_the_selected_user_home;

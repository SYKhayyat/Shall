pub use shall::verbs::perform_maintenance;
pub use shall::verbs::prelude::*;
pub use shall::verbs::sync::*;

fn spec(bare: bool) -> shall::core::PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if bare {
        options.set("__bare", "true");
    }
    shall::core::PackageSpec {
        name: "htop".into(),
        backend: "apt".into(),
        options,
        requires: Vec::new(),
        present: true,
    }
}
mod a_bare_install_is_asked_and_can_be_declined;
mod a_bare_install_refuses_without_a_terminal;
mod a_qualified_install_does_not_prompt;
mod explicit_confirmation_skips_the_bare_prompt;
mod no_bare_install_does_not_prompt;

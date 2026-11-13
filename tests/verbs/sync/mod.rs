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
mod a_never_decline_is_a_warning_and_not_a_failure;
mod a_qualified_install_does_not_prompt;
mod explicit_confirmation_skips_the_bare_prompt;
mod no_bare_install_does_not_prompt;
mod the_setting_can_answer_instead_of_the_prompt;
mod the_setting_is_read_by_the_unattended_run_too;

/// A fixture with `lines` written into a manifest, `[declare] install` set to `policy`, and
/// `--yes` on, so a run never stops at the interactive confirmation.
///
/// A bare line is resolved by asking the managers in `priority` whether they have the name, so the
/// fixture also states that this machine's apt catalogue has it (`#162`: stated, not read out of
/// the mock's silence).
async fn declaring(
    lines: &[&str],
    policy: shall::config::config::BareInstall,
) -> crate::mock_providers::TestKernel {
    use shall::model::Landing;

    let mut kernel = crate::mock_providers::TestKernel::new().await;
    for line in lines.iter().filter(|l| !l.contains(':')) {
        kernel.mock_executor.set_response(
            &format!("apt-cache search -- {line}"),
            Ok(shall::core::executor::DryRunOutput {
                stdout: format!("{line} - a program this machine has\n").into_bytes(),
                stderr: Vec::new(),
            }
            .into()),
        );
    }
    for line in lines {
        kernel
            .app
            .declarations()
            .declare(line, None, Landing::Imperative)
            .await
            .expect("the fixture's own declaration must be written for real");
    }
    let mut config = (*kernel.app.config).clone();
    config.declare.install = policy;
    config.yes = true;
    kernel.app.config = std::sync::Arc::new(config);
    kernel
}

use super::*;

/// `#94`'s second half, second reading: `[declare] install = "always"` is the answer given once
/// instead of every run.
#[test]
fn always_installs_a_bare_declaration_without_asking() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    let mut asked = false;
    let decision = confirm_bare_installs_with(
        &changes,
        shall::config::config::BareInstall::Always,
        false,
        |_| {
            asked = true;
            Ok(false)
        },
    )
    .unwrap();
    assert_eq!(decision, BareDecision::Proceed);
    assert!(
        !asked,
        "`always` means the question is not put to anyone, not that it is answered automatically \
         — a prompt that appears and answers itself is a prompt the user learns to distrust"
    );
}

/// And `never` is the third reading, which `--yes` cannot express: a flag can say "do it without
/// asking" and there is no flag whose meaning is "and do not do it either".
#[test]
fn never_installs_nothing_even_when_the_run_says_yes() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    let mut asked = false;
    let decision = confirm_bare_installs_with(
        &changes,
        shall::config::config::BareInstall::Never,
        true,
        |_| {
            asked = true;
            Ok(true)
        },
    )
    .unwrap();
    assert_eq!(decision, BareDecision::Declined(BareDecline::Setting));
    assert!(
        !asked,
        "`never` does not ask, so nothing may be put to the terminal"
    );
}

/// The declaration stays in the file, so the count of what is owed has to include it: a `never`
/// machine whose sync printed `0 left` would be a run reporting that it had converged.
#[test]
fn a_declined_setting_still_names_what_is_owed() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    assert_eq!(bare_install_names(&changes).len(), 1);
}

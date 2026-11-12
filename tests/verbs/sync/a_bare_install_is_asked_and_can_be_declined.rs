use super::*;

#[test]
fn a_bare_install_is_asked_and_can_be_declined() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    let mut prompt = None;
    let decision = confirm_bare_installs_with(
        &changes,
        shall::config::config::BareInstall::Ask,
        false,
        |text| {
            prompt = Some(text.to_string());
            Ok(false)
        },
    )
    .unwrap();
    assert_eq!(decision, BareDecision::Declined(BareDecline::Asked));
    assert_eq!(
        prompt.as_deref(),
        Some("Install apt:htop from a bare declaration?")
    );
}

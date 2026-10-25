use super::*;

/// **`snap info` answers from the store.** It prints publisher, summary and channels for
/// any snap that exists, installed or not — the fixture above already says so, and
/// `installed_state` already returns `None` for it. `info` was the caller that did not ask:
/// it returned `Some` for the whole report, so the planner read every declared snap as
/// already present and `shall install snap:x` reported success having installed nothing.
#[tokio::test]
async fn info_answers_installed_here_not_present_in_the_store() {
    let store_only = "name:      code\n\
                      summary:   Code editing. Redefined.\n\
                      publisher: Snapcrafters\n\
                      channels:\n  \
                      latest/stable:    1.85.1\n";
    let (core, _) = snap_with("snap info -- code", store_only);
    assert!(
        SnapQueryable { core }.info("code").await.unwrap().is_none(),
        "a snap that only exists in the store was reported as installed"
    );

    let on_the_machine = "name:      code\n\
                          tracking:     latest/stable\n\
                          installed:          1.85.1                     (139) 351MB classic\n";
    let (core, _) = snap_with("snap info -- code", on_the_machine);
    let found = SnapQueryable { core }
        .info("code")
        .await
        .unwrap()
        .expect("installed");
    assert_eq!(found.version.as_deref(), Some("1.85.1"));
    assert_eq!(
        found.properties.get("classic").map(String::as_str),
        Some("true"),
        "Q20's confinement still reaches the planner"
    );
    assert_eq!(
        found.properties.get("channel").map(String::as_str),
        Some("stable"),
        "and D13's channel with it"
    );
}

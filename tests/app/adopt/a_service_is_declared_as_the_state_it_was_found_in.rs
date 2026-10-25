use super::*;

#[test]
fn a_service_is_declared_as_the_state_it_was_found_in() {
    // A bare `service:AppMgmt` means enable AND start, and enabling on Windows rewrites the
    // start type to automatic. The init only reports RUNNING services, so that is the only
    // half that was observed — adopting the other half would reconfigure boot on the first
    // sync after a command whose promise is to describe the machine as it already is.
    let options = HashMap::from([(
        "service".to_string(),
        vec![("status".to_string(), "running".to_string())],
    )]);
    let found = Discovery {
        adopt: vec![
            Package::new("AppMgmt", "service"),
            Package::new("jq", "apt"),
        ],
        ..Default::default()
    };
    let m = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
    );
    let text = m.render_manifest(&found, &options);

    assert!(
        text.contains("\nservice:AppMgmt@status=running\n"),
        "the observed state, and only the observed state:\n{text}"
    );
    assert!(
        !text.contains("enabled="),
        "the start type was never looked at, so no line may claim it:\n{text}"
    );
    // A package carries no options, because its name already said everything the listing did.
    assert!(text.contains("\napt:jq\n"), "{text}");
}

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
        quiet_executor(),
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

/// A unit found in the invoking account's own domain is written into that domain (`#211`).
///
/// The discovery marks the candidate; without the mark reaching the line, `adopt` would write
/// `service:gpg-agent@status=running` and the first `sync` would enable and start that unit for
/// the whole machine — a different service from the one that was adopted, under the same name.
#[test]
fn a_unit_found_in_an_accounts_domain_is_declared_in_it() {
    let options = HashMap::from([(
        "service".to_string(),
        vec![("status".to_string(), "running".to_string())],
    )]);
    let mut mine = Package::new("gpg-agent", "service");
    mine.properties
        .insert("user".to_string(), "alice".to_string());
    let found = Discovery {
        adopt: vec![mine, Package::new("sshd", "service")],
        ..Default::default()
    };
    let m = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
        quiet_executor(),
    );
    let text = m.render_manifest(&found, &options);

    assert!(
        text.contains("\nservice:gpg-agent@status=running,user=alice\n"),
        "the account's own unit says whose it is:\n{text}"
    );
    assert!(
        text.contains("\nservice:sshd@status=running\n"),
        "a system unit carries no account mark at all:\n{text}"
    );
}

/// A unit's owning package is written beside the line as a comment (`#211`).
///
/// The grammar has always read a `#` after whitespace as a comment, so the resource gains no new
/// option and no new parse: the comment is appended after the round trip that decided the line,
/// which is why a name that needed quoting still gets its line and its comment.
#[test]
fn a_unit_says_which_package_installed_it() {
    let options = HashMap::from([(
        "service".to_string(),
        vec![("status".to_string(), "running".to_string())],
    )]);
    let mut unit = Package::new("sshd", "service");
    unit.properties
        .insert("owned_by".to_string(), "openssh-server".to_string());
    let found = Discovery {
        adopt: vec![unit, Package::new("cron", "service")],
        ..Default::default()
    };
    let m = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
        quiet_executor(),
    );
    let text = m.render_manifest(&found, &options);

    assert!(
        text.contains("\nservice:sshd@status=running  # installed by openssh-server\n"),
        "the line, then the package that would put it back:\n{text}"
    );
    assert!(
        text.contains("\nservice:cron@status=running\n"),
        "a unit no manager claimed gets no comment — an unverifiable name is worse than none:\n{text}"
    );
}

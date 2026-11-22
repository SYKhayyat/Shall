// The `ufw` row is Linux-only and `detect` asks the running OS which rows apply (the table's
// `os` column), so this drives on Linux only. The parsing half it rests on is not gated:
// `backends::firewall::a_default_policy_is_read_from_the_listing_that_states_it` runs everywhere,
// and on a machine with no row at all `adopt` reports "no firewall Shall knows is running".
#![cfg(target_os = "linux")]

use super::*;

/// Real `ufw status verbose` output — the row's own `list` command.
const UFW: &str = "Status: active\n\
     Logging: on (low)\n\
     Default: deny (incoming), allow (outgoing), disabled (routed)\n\
     New profiles: skip\n\
     \n\
          To                         Action      From\n\
          --                         ------      ----\n\
     22/tcp                     ALLOW IN    Anywhere\n\
     8080/udp                   ALLOW IN    Anywhere\n";

fn adopting_ufw() -> (Adopter, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_command_exists("ufw", true);
    mock.set_response(
        // `sudo`, because reading a perimeter is a privileged read — the same call, with the
        // same non-interactive form, that the phase which applies one makes.
        "sudo -n ufw status verbose",
        Ok(DryRunOutput {
            stdout: UFW.as_bytes().to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let config = Config::default();
    let m = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        Arc::new(Mutex::new(StateRegistry::default())),
        &config,
        exec,
    );
    (m, mock)
}

fn scope(backends: &[&str], added_only: bool, inbound: bool) -> AdoptScope {
    AdoptScope {
        backends: backends.iter().map(|s| s.to_string()).collect(),
        enabled_only: false,
        added_only,
        inbound,
        all_services: false,
        schemas: Vec::new(),
        user_added: false,
        no_managers: true,
    }
}

/// C39, against II.99: **`adopt` offers everything the machine can name, and every narrowing is
/// a flag the person passes.** Three claims, one family:
///
/// - the rules in force are offered, ports and default policies both;
/// - a default policy is offered with the value it was found with, because a policy without one
///   declares nothing — and the option is per rule, which is why it cannot ride `adopt`'s
///   one-set-per-backend option map;
/// - the narrowings narrow, and the default is the whole perimeter.
#[tokio::test]
async fn a_firewall_rule_is_declared_as_the_perimeter_it_enforces() {
    let (m, _) = adopting_ufw();
    let found = m
        .discover_scoped(&scope(&["firewall"], false, false))
        .await
        .unwrap();
    let names: Vec<&str> = found
        .firewall
        .iter()
        .map(|r| r.package.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["22/tcp", "8080/udp", "default/incoming", "default/outgoing"],
        "ports and both policies, sorted so the manifest diffs cleanly"
    );

    let text = m.render_manifest(&found, &HashMap::new());
    assert!(text.contains("\nfirewall:22/tcp\n"), "{text}");
    assert!(text.contains("\nfirewall:8080/udp\n"), "{text}");
    assert!(
        text.contains("\nfirewall:default/incoming@value=deny\n"),
        "the policy it was found with, not one Shall prefers:\n{text}"
    );
    assert!(
        text.contains("\nfirewall:default/outgoing@value=allow\n"),
        "the outgoing policy carries its own value — one option per rule:\n{text}"
    );
    assert!(
        text.contains("ufw status verbose"),
        "the manifest names the command behind the answer:\n{text}"
    );

    // `--added-only`: what somebody added, not the baseline the firewall shipped with.
    let found = m
        .discover_scoped(&scope(&["firewall"], true, false))
        .await
        .unwrap();
    let names: Vec<&str> = found
        .firewall
        .iter()
        .map(|r| r.package.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["22/tcp", "8080/udp"],
        "--added-only drops the policies"
    );

    // `--inbound`: the incoming side. It narrows the policies and only them, because a
    // `firewall:` port carries no direction at all.
    let found = m
        .discover_scoped(&scope(&["firewall"], false, true))
        .await
        .unwrap();
    let names: Vec<&str> = found
        .firewall
        .iter()
        .map(|r| r.package.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["22/tcp", "8080/udp", "default/incoming"],
        "--inbound keeps every port and drops the outgoing policy"
    );
}

/// An unreadable perimeter is not an empty one. The whole ruling rests on a manifest being
/// honest about what it could not see (II.99's third refusal), and "no rules found" is exactly
/// the sentence that turns a firewall nobody could read into a machine with no open ports.
#[tokio::test]
async fn a_perimeter_that_cannot_be_read_is_reported_rather_than_offered_empty() {
    let (m, mock) = adopting_ufw();
    mock.set_response(
        // `sudo`, because reading a perimeter is a privileged read — the same call, with the
        // same non-interactive form, that the phase which applies one makes.
        "sudo -n ufw status verbose",
        Err(shall::core::Error::command_failed("permission denied")),
    );
    let found = m
        .discover_scoped(&scope(&["firewall"], false, false))
        .await
        .unwrap();
    assert!(found.firewall.is_empty(), "nothing is offered");
    let reason = found
        .skipped_backends
        .iter()
        .find(|(who, _)| who == "firewall")
        .map(|(_, why)| why.clone())
        .expect("the source is named with its reason");
    assert!(
        reason.contains("could not be read") && reason.contains("permission denied"),
        "the reason names the command and the failure: {reason}"
    );
    let text = m.render_manifest(&found, &HashMap::new());
    assert!(
        !text.contains("The perimeter this machine enforces"),
        "an empty reading gets no section at all:\n{text}"
    );
}

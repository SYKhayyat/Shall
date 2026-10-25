use super::*;

/// XI.2: one spelling, several firewalls. The same declaration produces each tool's own
/// command — which is the argument for a built-in backend over a per-machine definition.
#[test]
fn one_declaration_becomes_each_firewalls_own_command() {
    let ufw = shipped("ufw").allow_command(22, Proto::Tcp);
    assert_eq!(ufw, vec!["ufw", "allow", "22/tcp"]);

    let fd = shipped("firewalld").allow_command(22, Proto::Tcp);
    assert_eq!(fd, vec!["firewall-cmd", "--permanent", "--add-port=22/tcp"]);

    let win = shipped("windows-defender").allow_command(22, Proto::Tcp);
    assert!(win.iter().any(|a| a == "localport=22"), "{:?}", win);
    assert!(win.iter().any(|a| a == "protocol=tcp"), "{:?}", win);
}

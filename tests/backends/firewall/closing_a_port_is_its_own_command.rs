use super::*;

#[test]
fn closing_a_port_is_its_own_command() {
    assert_eq!(
        shipped("ufw").deny_command(8080, Proto::Udp),
        vec!["ufw", "delete", "allow", "8080/udp"]
    );
}

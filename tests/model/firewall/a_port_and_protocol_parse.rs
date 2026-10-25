use super::*;

#[test]
fn a_port_and_protocol_parse() {
    assert_eq!(Rule::parse("22/tcp"), Ok(port(22)));
    assert_eq!(
        Rule::parse("53/udp"),
        Ok(Rule::Port {
            port: 53,
            proto: Proto::Udp
        })
    );
    assert_eq!(Rule::parse(" 8080 / TCP "), Ok(port(8080)));
}

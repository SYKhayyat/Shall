use super::*;

#[test]
fn the_refusal_names_the_surface_the_file_and_how_a_row_opens() {
    let msg = cannot_use(surface("firewall").unwrap(), "missing field `name`");
    assert!(msg.contains("adapters/firewall.toml"));
    assert!(msg.contains("[[firewall]]"));
    assert!(msg.contains("missing field `name`"));
    assert!(msg.contains("shall adapters"));
}

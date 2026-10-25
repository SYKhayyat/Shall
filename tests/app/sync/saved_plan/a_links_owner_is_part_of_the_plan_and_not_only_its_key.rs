use super::*;

#[test]
fn a_links_owner_is_part_of_the_plan_and_not_only_its_key() {
    let path = "/etc/ssh/ssh_config";
    let alice = resources_for(&link(path, &[("owner", "alice")]));
    let bob = resources_for(&link(path, &[("owner", "bob")]));

    assert_eq!(alice.place, bob.place, "the owner widened the key");
    assert_ne!(artifact(&alice), artifact(&bob));
}

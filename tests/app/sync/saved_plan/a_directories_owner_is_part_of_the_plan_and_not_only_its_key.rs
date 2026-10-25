use super::*;

#[test]
fn a_directories_owner_is_part_of_the_plan_and_not_only_its_key() {
    let path = an_absolute_dir();
    let alice = resources_for(&dir(path, &[("owner", "alice")]));
    let bob = resources_for(&dir(path, &[("owner", "bob")]));

    assert_eq!(alice.place, bob.place, "the owner widened the key");
    assert_ne!(artifact(&alice), artifact(&bob));
}

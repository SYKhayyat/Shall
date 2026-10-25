use super::*;

#[test]
fn a_directories_mode_is_part_of_the_plan_and_not_only_its_key() {
    let path = an_absolute_dir();
    let tight = resources_for(&dir(path, &[("mode", "0700")]));
    let loose = resources_for(&dir(path, &[("mode", "0750")]));

    assert_eq!(
        tight.place, loose.place,
        "the mode is not the key's business, and the key must not widen to hold it: the \
         ledger addresses a directory by where it is"
    );
    assert_ne!(
        tight.intent_attributes(tight.place[0].as_str()),
        loose.intent_attributes(loose.place[0].as_str()),
        "0700 and 0750 froze the same attributes"
    );
    assert_ne!(
        hash_plan(&[], &[], &tight),
        hash_plan(&[], &[], &loose),
        "a plan that cannot tell 0700 from 0750 passes its own drift check on a changed mode"
    );
    assert_ne!(artifact(&tight), artifact(&loose));
}

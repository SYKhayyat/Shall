use super::*;

#[test]
fn a_bare_name_defers_the_capability_check_to_the_resolver() {
    // No prefix means `priority` decides the backend, so the grammar cannot know yet
    // whether `formats` is legal — refusing here would break every unprefixed line.
    assert!(p("fd@formats=deb").is_ok());
}

use super::*;

#[test]
fn the_reserved_words_are_the_ones_the_prefix_grammar_spends() {
    // `re:` introduces a pattern and `list` names the priority file, so a backend
    // answering to either would make `re:foo` / `apt,list:foo` ambiguous forever. The
    // onboarder refuses these names; this pins the list it refuses.
    assert!(RESERVED_BACKEND_NAMES.contains(&"re"));
    assert!(RESERVED_BACKEND_NAMES.contains(&PRIORITY_KEYWORD));
}

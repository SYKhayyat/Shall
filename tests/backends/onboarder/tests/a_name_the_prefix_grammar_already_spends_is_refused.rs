use super::*;

#[test]
fn a_name_the_prefix_grammar_already_spends_is_refused() {
    // `re` and `list` mean something in a `backend:name` prefix, so a backend answering
    // to one could never be reached by a line: `list:rg` would keep meaning the priority
    // file. Refusing the name is the only place that can be said out loud.
    assert!(!is_valid_backend_name("re"));
    assert!(!is_valid_backend_name("list"));
    // A comma splits a chain and a colon splits the prefix, so neither can be in a name.
    assert!(!is_valid_backend_name("apt,dnf"));
    assert!(!is_valid_backend_name("we:ird"));
    // A hyphen is fine, and has to be: `nix-env` and `apt-get` are real names, which is
    // why a chain is comma-separated.
    assert!(is_valid_backend_name("nix-env"));
}

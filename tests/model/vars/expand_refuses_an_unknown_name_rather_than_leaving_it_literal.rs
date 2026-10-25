use super::*;

#[test]
fn expand_refuses_an_unknown_name_rather_than_leaving_it_literal() {
    let facts = expand_facts();
    let err = expand("~/.config/$rle/init.lua", &facts, &origin(3)).unwrap_err();
    assert!(err.what.contains("rle"), "{}", err);
}

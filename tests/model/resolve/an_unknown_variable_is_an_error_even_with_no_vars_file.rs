use super::*;

/// With no `vars` file the variable set is empty, and an empty set used to mean "skip
/// expansion entirely" — so `$role` survived as literal text and became a path with a
/// dollar in it, failing later and somewhere else. It has to be the same error here that
/// a misspelled name is when the file does exist.
#[test]
fn an_unknown_variable_is_an_error_even_with_no_vars_file() {
    let f = fx(
        "Work\n",
        &[("Work", "use m\n")],
        &[("m.txt", "link:~/.config/$role/init.lua\n")],
    );
    let err = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap_err()
        .to_string();
    assert!(err.contains("role"), "{}", err);
}

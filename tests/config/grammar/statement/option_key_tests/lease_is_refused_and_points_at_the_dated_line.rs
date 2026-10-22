use super::*;

#[test]
fn lease_is_refused_and_points_at_the_dated_line() {
    // S19. II.16 retired `@lease=2h`, and nothing Shall writes used it — but
    // `StateRegistry::add` still READ it and turned it into a real expiry, so a
    // hand-written lease was silently a package that uninstalls itself, on a path the
    // guard does not see (C3).
    let err = parse_line("apt:jq@lease=2h").unwrap_err();
    assert!(err.what.contains("`@lease` is not an option"), "{}", err);
    assert!(
        err.hint.unwrap().contains("@expires="),
        "must teach the replacement"
    );
}

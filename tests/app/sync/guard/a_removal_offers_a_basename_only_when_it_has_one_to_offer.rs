use super::*;

/// The identities a `protected_packages` rule is offered, and no others.
///
/// **Asserted on the list, because the caller cannot see it.** `inspect_removals` runs
/// `find_map` over the result, so a redundant candidate is invisible there: a duplicate name
/// answers the same question twice and an empty string matches no rule. Both guards in this
/// function are therefore unfalsifiable one level up â€” which is exactly how
/// `base != name && !base.is_empty()` came to survive being read as `||`.
#[test]
fn a_removal_offers_a_basename_only_when_it_has_one_to_offer() {
    // A package contributes its name and nothing else, whatever the name looks like. A
    // `link:` path and an `apt:` package can be the same string and mean different things.
    assert_eq!(
        protected_names(RemovalKind::Package, "/home/u/.vimrc"),
        ["/home/u/.vimrc"]
    );
    // A port is not a path: `22/tcp` splits into something, and `tcp` is not an identity
    // anybody wrote a rule about.
    assert_eq!(protected_names(RemovalKind::Port, "22/tcp"), ["22/tcp"]);

    // An extra whose identity is a path contributes both, on either separator, so a user may
    // name the thing rather than the absolute path Shall happens to key it by.
    assert_eq!(
        protected_names(RemovalKind::Extra, "/home/u/.vimrc"),
        ["/home/u/.vimrc", ".vimrc"]
    );
    assert_eq!(
        protected_names(RemovalKind::Extra, "C:\\Users\\u\\vimrc"),
        ["C:\\Users\\u\\vimrc", "vimrc"]
    );

    // An extra that is already a bare name contributes it once.
    assert_eq!(protected_names(RemovalKind::Extra, "vimrc"), ["vimrc"]);
    // And one ending in a separator has no basename to offer, rather than an empty one.
    assert_eq!(
        protected_names(RemovalKind::Extra, "/home/u/"),
        ["/home/u/"]
    );
}

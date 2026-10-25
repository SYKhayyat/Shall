use super::*;

/// **What is written is what is read back.** The ledger is a set of these strings on disk,
/// so `Display` and `FromStr` being inverses is the whole of its wire format.
///
/// A `repo:` subject carries its own colons, which is why the split is at the FIRST one and
/// why the subject is not split again here: `repo:apt:ppa:x/y` must hand the undo
/// `apt:ppa:x/y`, whole.
#[test]
fn a_key_round_trips_through_the_string_the_ledger_stores() {
    for (text, kind, subject) in [
        ("service:nginx", ResourceKind::Service, "nginx"),
        ("repo:apt:ppa:x/y", ResourceKind::Repo, "apt:ppa:x/y"),
        ("link:/home/u/.vimrc", ResourceKind::Link, "/home/u/.vimrc"),
        ("firewall:22/tcp", ResourceKind::Firewall, "22/tcp"),
    ] {
        let parsed: ExtraKey = text.parse().unwrap_or_else(|_| panic!("`{text}` parses"));
        assert_eq!(parsed, ExtraKey::new(kind, subject));
        assert_eq!(parsed.to_string(), text);
    }

    // A package key is not an extras key, which is the confusion `Statement::kind`'s own
    // doc comment warns about: re-splitting on `:` would read `apt:jq` as the kind `apt`.
    assert_eq!("apt:jq".parse::<ExtraKey>(), Err(()));
    // And a bare word names no kind at all.
    assert_eq!("nginx".parse::<ExtraKey>(), Err(()));
}

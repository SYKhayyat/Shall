use super::*;

#[test]
fn the_options_a_key_already_carries_are_not_repeated_in_the_fingerprint() {
    let account = shall::core::account::Account::current()
        .expect("the invoking account has a passwd entry")
        .name()
        .to_string();
    for (statement, key_fragments, uncarried) in [
        (
            {
                let mut opts = shall::config::grammar::Options::default();
                opts.set("value", "prefer-dark");
                opts.set("scope", "system");
                opts.set("user", &account);
                opts.set("owner", "the-owner");
                shall::config::grammar::Statement::Setting("org.gnome.x/theme".into(), opts)
            },
            vec!["@scope=system".to_string(), format!("@user={account}")],
            vec!["owner=the-owner"],
        ),
        (
            {
                let mut opts = shall::config::grammar::Options::default();
                opts.set("target", "/etc/motd");
                opts.set("owner", "the-owner");
                opts.set("user", &account);
                shall::config::grammar::Statement::Link("source".into(), opts)
            },
            vec!["/etc/motd".to_string()],
            vec!["owner=the-owner"],
        ),
    ] {
        let resources = resources_for(&statement);
        let key = &resources.place[0];
        let attributes = resources.intent_attributes(key.as_str());
        for fragment in &key_fragments {
            assert!(key.contains(fragment), "{fragment} is not in the key {key}");
        }
        for carried in ["scope", "user", "target"] {
            assert!(
                !attributes
                    .iter()
                    .any(|a| a.starts_with(&format!("{carried}="))),
                "{carried}= is in the key already, and the fingerprint repeated it: {attributes:?}"
            );
        }
        for expected in &uncarried {
            assert!(
                attributes.iter().any(|a| a == expected),
                "the option the key does not carry is missing: {expected:?} not in {attributes:?}"
            );
        }
    }
}

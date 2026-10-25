use super::*;

/// An extra is reported by kind and subject, not by re-splitting its key on `:` — which
/// `firewall:22/tcp` and `setting:org.gnome.desktop/x` would both survive, and
/// `repo:apt:ppa:x/y` would not.
#[test]
fn extras_split_into_kind_and_subject() {
    use shall::config::grammar::{Options, Statement};
    let state = shall::model::DesiredState {
        extras: vec![
            (
                Statement::Firewall("22/tcp".into(), Options::default()),
                Origin::new("/repo/modules/net.txt", 3),
            ),
            (
                Statement::Repo {
                    backend: "apt".into(),
                    spec: "ppa:x/y".into(),
                },
                Origin::new("/repo/modules/net.txt", 1),
            ),
        ],
        ..Default::default()
    };
    let doc = Evaluation::of(&state, Path::new("/repo"));
    let by_kind = |k: &str| {
        doc.extras
            .iter()
            .find(|e| e.kind == k)
            .unwrap_or_else(|| panic!("{} missing from {:?}", k, doc.extras))
    };
    assert_eq!(by_kind("firewall").name, "22/tcp");
    assert_eq!(by_kind("repo").name, "apt:ppa:x/y");
    assert_eq!(by_kind("repo").source, "modules/net.txt:1");
}

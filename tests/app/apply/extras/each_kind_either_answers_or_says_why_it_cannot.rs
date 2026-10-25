use super::*;

/// **Which kinds the probe can answer, stated as an assertion rather than as a `_` arm.**
///
/// `None` means *unverifiable*, and unverifiable **places** — so a kind falling through
/// here is re-applied on every sync for ever. That used to be a `_ => None` at the bottom
/// of a `match` on a `&str`, where a kind arrived by omission and nobody had to say why.
/// The dispatch is exhaustive over `ResourceKind` now; this pins the answers it gives, so
/// changing one is a visible decision.
#[tokio::test]
async fn each_kind_either_answers_or_says_why_it_cannot() {
    let config = Arc::new(shall::config::Config::default());
    let reg = BackendRegistry::new();
    let exec = CommandExecutor::new(false, false);
    let mut opts = Options::default();
    opts.insert("value", "1");

    for (stmt, key) in [
        (
            Statement::Repo {
                backend: "apt".into(),
                spec: "ppa:x/y".into(),
            },
            "repo:apt:ppa:x/y",
        ),
        (
            Statement::Schedule("nightly".into(), opts.clone()),
            "schedule:nightly",
        ),
        (
            Statement::Firewall("22/tcp".into(), opts.clone()),
            "firewall:22/tcp",
        ),
    ] {
        let key: ExtraKey = key.parse().expect("the fixture keys are well formed");
        assert_eq!(
            in_effect(&config, &reg, &exec, &stmt, &key).await,
            None,
            "`{key}` is documented as unverifiable; if that changed, say so here"
        );
    }

    // `setting:` is the one that left this list (J2). It answers from the store now, and
    // what it still cannot answer it reports as unanswerable rather than as absent: a key
    // whose name is not `SCHEMA/KEY` addresses nothing, and a store this machine does not
    // run has no value to compare. Both place, which is what they did before.
    let key: ExtraKey = "setting:dark".parse().expect("well formed");
    assert_eq!(
        in_effect(
            &config,
            &reg,
            &exec,
            &Statement::Setting("dark".into(), opts.clone()),
            &key
        )
        .await,
        None,
        "`dark` is not `SCHEMA/KEY`, so it addresses no key in any store"
    );
    // A line with no `@value=` is invalid and the installer refuses it by name. Answering
    // here would be answering for a declaration that is never going to apply.
    let key: ExtraKey = "setting:org.gnome.x/theme".parse().expect("well formed");
    assert_eq!(
        in_effect(
            &config,
            &reg,
            &exec,
            &Statement::Setting("org.gnome.x/theme".into(), Options::default()),
            &key
        )
        .await,
        None,
    );

    // A key whose kind is not a keyword at all — a package line's `backend:name` — cannot
    // even be built now, which is the point of the type: `ExtraKey` refuses it at the parse.
    assert_eq!("apt:jq".parse::<ExtraKey>(), Err(()));
}

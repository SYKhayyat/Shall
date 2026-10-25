use super::*;

/// **The window `S87` lives in, and what closes it.**
///
/// The log is written per operation; the ownership registry is written **once, at the end of
/// a run**. A kill in between leaves a package installed, its install recorded here as
/// `Completed`, and nothing in the registry claiming it — owned by nobody. Nothing
/// downstream notices: `sync` converges (the package genuinely is installed), the preview
/// plans nothing (there is nothing to plan), and the damage surfaces only when somebody
/// tries to remove it, as a cleanup that reports success and takes nothing away.
///
/// `interrupted_actions` cannot see these — the entry is **closed**, not open — which is
/// exactly why recovery walked past them. `reconcile_ownership` reads this instead, so a
/// declared package this machine's own log records installing is claimed on that evidence
/// rather than on whether the manager's listing happens to report it today.
///
/// That last clause is the half a container found: a `SIGKILL` can leave a package
/// **unpacked but not configured** — on disk, on `PATH`, and correctly reported by
/// `dpkg-query`'s status field as *not installed*. The lister is right to say so. Ownership
/// is a different question from installedness, and this is the answer to it.
#[tokio::test]
async fn a_completed_install_is_recorded_even_though_it_is_not_interrupted() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("j.jsonl");
    let journal = tokio::sync::Mutex::new(Journal::at(path.clone()).unwrap());

    // One install that finished, one that was killed part-way, one removal that finished.
    journalled(
        &journal,
        vec![JournalAction::Install(a_spec("landed"))],
        std::future::ready(Ok(())),
    )
    .await
    .unwrap();
    {
        let pending = journalled(
            &journal,
            vec![JournalAction::Install(a_spec("half"))],
            std::future::pending::<Result<()>>(),
        );
        futures::pin_mut!(pending);
        let _ = futures::poll!(&mut pending);
    }
    journalled(
        &journal,
        vec![JournalAction::Remove {
            name: "gone".into(),
            backend: "apt".into(),
        }],
        std::future::ready(Ok(())),
    )
    .await
    .unwrap();

    let after = Journal::at(path).unwrap();

    let completed = after.completed_installs();
    assert_eq!(
        completed,
        vec![("apt".to_string(), "landed".to_string())],
        "only the install that finished belongs here — not the interrupted one, which \
         `interrupted_actions` owns, and not the removal, which takes ownership away rather \
         than granting it"
    );

    // The control, and it is the whole reason this method had to exist: the entry above is
    // invisible to the mechanism recovery already had.
    let interrupted: Vec<String> = after
        .interrupted_actions()
        .iter()
        .map(|e| e.action.key())
        .collect();
    assert_eq!(
        interrupted,
        vec!["apt:half".to_string()],
        "a completed install is not interrupted, so recovery's existing reader cannot see \
         it — which is why a package it left behind stayed owned by nobody"
    );
}

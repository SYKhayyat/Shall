use super::*;

/// **The names are interpolated into evaluated Lisp, so one bad name must stop the whole
/// batch** — not fail after the good ones have already been written into the form. This is
/// the property batching could most easily have lost.
#[tokio::test]
async fn one_illegal_name_refuses_the_whole_batch_before_anything_runs() {
    let (core, mock) = wired();
    let specs = vec![
        shall::core::PackageSpec {
            name: "csv-mode".into(),
            backend: "emacs".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "evil (shell-command \"rm -rf /\")".into(),
            backend: "emacs".into(),
            ..Default::default()
        },
    ];
    let err = EmacsInstallable { core }.install(&specs, false).await;
    assert!(err.is_err(), "an illegal symbol must refuse the batch");
    assert!(
        mock.get_calls().await.is_empty(),
        "nothing may run when any name in the batch is refused"
    );
}

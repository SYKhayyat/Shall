use super::*;

#[tokio::test]
async fn a_backend_missing_from_priority_is_refused_by_name() {
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use base\n"),
        ("modules/base.txt", "snap:foo\n"),
    ]);
    let err = resolve(&r).await.unwrap_err().to_string();
    // Two refusals guard this, and which one fires depends on whether the backend is
    // one Shall has ever heard of: the grammar refuses a prefix that names nothing,
    // `priority` refuses a real backend you did not list (V.15). Both must name the
    // backend, point at `priority`, and say where the line is — an error that cannot
    // be located cannot be fixed.
    assert!(err.contains("snap"), "{}", err);
    assert!(err.contains("priority"), "{}", err);
    assert!(err.contains("base.txt:1"), "{}", err);
    // Never silently dropped, which is what the old resolver did with a backend it
    // did not recognise.
    assert!(!err.is_empty());
}

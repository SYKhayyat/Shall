use super::*;

/// The J2 fix, in one assertion: the reporting half asks the store the same question the
/// installer has always asked it. Before this, every `setting:` line was *unverifiable* —
/// which places — so `check` and `plan` named a settled key as work on every run.
#[tokio::test]
async fn a_key_the_store_already_holds_reads_back_as_in_effect() {
    let core = store_reading("prefer-dark");
    assert_eq!(
        core.holds(
            "org.gnome.desktop.interface/color-scheme",
            "prefer-dark",
            None,
            None
        )
        .await,
        Some(true)
    );
    assert_eq!(
        core.holds(
            "org.gnome.desktop.interface/color-scheme",
            "prefer-light",
            None,
            None
        )
        .await,
        Some(false),
        "a different value is drift, and drift places"
    );
}

use super::*;

/// An app already on the declared branch is not switched, and — the reason the listing is
/// read at all — an app on *two* branches is not either: nothing says which one runs, so
/// nothing here may claim to know (D13).
#[tokio::test]
async fn an_unreadable_or_matching_branch_issues_no_switch() {
    for listing in [
        "org.gimp.GIMP\t2.10\tbeta\n",
        "org.gimp.GIMP\t2.10\tstable\norg.gimp.GIMP\t2.99\tbeta\n",
    ] {
        let (core, mock) = scripted(listing);
        FlatpakInstallable { core }
            .install(&[spec_with("org.gimp.GIMP", &[("channel", "beta")])], false)
            .await
            .unwrap();
        assert!(
            !mock
                .get_calls()
                .await
                .iter()
                .any(|c| c.contains("make-current")),
            "listing {listing:?} produced a switch"
        );
    }
}

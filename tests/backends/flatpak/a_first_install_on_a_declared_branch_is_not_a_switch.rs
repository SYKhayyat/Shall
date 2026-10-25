use super::*;

/// The first install of an app is not a switch: flatpak has only the one branch to run, and
/// a `make-current` on every install would be a second command doing nothing.
#[tokio::test]
async fn a_first_install_on_a_declared_branch_is_not_a_switch() {
    let (core, mock) = scripted("");
    FlatpakInstallable { core }
        .install(&[spec_with("org.gimp.GIMP", &[("channel", "beta")])], false)
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec![
            "flatpak list --app --columns=application,version,branch",
            "flatpak install --system -y --noninteractive --or-update -- org.gimp.GIMP//beta",
        ]
    );
}

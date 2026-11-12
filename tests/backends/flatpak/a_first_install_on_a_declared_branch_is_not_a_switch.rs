use super::*;
use shall::core::executor::DryRunOutput;

/// The first install of an app is not a switch: flatpak has only the one branch to run, and
/// a `make-current` on every install would be a second command doing nothing.
#[tokio::test]
async fn a_first_install_on_a_declared_branch_is_not_a_switch() {
    let (core, mock) = scripted("");

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "flatpak install --system -y --noninteractive --or-update -- org.gimp.GIMP//beta",
        Ok(DryRunOutput::default().into()),
    );
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

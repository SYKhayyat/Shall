use super::*;
use shall::core::executor::DryRunOutput;

/// **flatpak has no channel switch.** Installing `app//beta` beside `app//stable` moves
/// nothing — flatpak keeps both, and the launcher still runs the branch it ran yesterday.
/// Without `make-current` the sync reports a repaired channel over a machine that changed
/// nothing a user could see.
#[tokio::test]
async fn switching_branch_installs_the_ref_and_then_points_the_app_at_it() {
    let (core, mock) = scripted("org.gimp.GIMP\t2.10\tstable\n");

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "flatpak install --system -y --noninteractive --or-update -- org.gimp.GIMP//beta",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "flatpak make-current --system -- org.gimp.GIMP beta",
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
            "flatpak make-current --system -- org.gimp.GIMP beta",
        ]
    );
}

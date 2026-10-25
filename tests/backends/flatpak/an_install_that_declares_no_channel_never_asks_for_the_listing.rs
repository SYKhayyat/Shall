use super::*;

/// A plan with no `@channel` in it does not read the listing at all — the branch question is
/// not asked of packages that never raised it.
#[tokio::test]
async fn an_install_that_declares_no_channel_never_asks_for_the_listing() {
    // No listing stub on purpose: registering one and asserting it went unused is what the
    // mock calls a belief the product disagreed with, so the absence *is* the assertion.
    let (core, mock) = scripted_without_a_listing();
    FlatpakInstallable { core }
        .install(&[spec_with("org.blender.Blender", &[])], false)
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec!["flatpak install --system -y --noninteractive --or-update -- org.blender.Blender"]
    );
}

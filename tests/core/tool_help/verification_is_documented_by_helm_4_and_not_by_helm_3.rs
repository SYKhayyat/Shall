use super::*;

/// The discriminator, against both real versions' help — captured from the tools, and the
/// only place the helm 3 arm can be asserted on a machine that has helm 4.
///
/// helm 3's help carries `--kube-insecure-skip-tls-verify`, which contains the word and is
/// not the flag; helm 4's carries `--verify` and `--keyring`. If this predicate answered
/// yes for helm 3 the gate would call a correct no-op drift, and if it answered no for
/// helm 4 a renamed flag would pass unnoticed — the two failures are opposite and the
/// fixtures pin both.
#[test]
fn verification_is_documented_by_helm_4_and_not_by_helm_3() {
    const V3: &str = include_str!("../../../tests/fixtures/helm/plugin-install-help-v3.txt");
    const V4: &str = include_str!("../../../tests/fixtures/helm/plugin-install-help-v4.txt");
    assert!(
        !help_documents_verification(V3),
        "helm 3 does not verify plugins; `--kube-insecure-skip-tls-verify` is not a \
         verification flag for the plugin being installed"
    );
    assert!(
        help_documents_verification(V4),
        "helm 4 documents `--verify` and `--keyring`"
    );
}

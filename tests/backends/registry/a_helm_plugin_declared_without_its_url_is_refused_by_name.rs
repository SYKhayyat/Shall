use super::*;

/// U39, at the wiring rather than in `generic`: the registered `helm` is the one that has
/// to refuse a plugin declared without its source, because a plugin installed under the
/// wrong identity is one nothing can remove afterwards.
#[tokio::test]
async fn a_helm_plugin_declared_without_its_url_is_refused_by_name() {
    let reg = build_registry().await;
    let helm = reg.get("helm").expect("helm is registered");
    let inst = helm.as_installable().expect("helm installs");
    let spec = shall::core::PackageSpec {
        name: "diff".into(),
        backend: "helm".into(),
        ..Default::default()
    };
    let msg = inst.install(&[spec], false).await.unwrap_err().to_string();
    assert!(msg.contains("helm:diff@url="), "{}", msg);
}

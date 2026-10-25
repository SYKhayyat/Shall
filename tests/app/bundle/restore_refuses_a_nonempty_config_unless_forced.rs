use super::*;

#[tokio::test]
async fn restore_refuses_a_nonempty_config_unless_forced() {
    let tmp = std::env::temp_dir().join(format!("shall-restore-{}", std::process::id()));
    let bundle = tmp.join("bundle");
    let cfg = tmp.join("cfg");
    let data = tmp.join("data");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(bundle.join("modules")).unwrap();
    std::fs::write(bundle.join("modules/tools.txt"), "apt:jq\n").unwrap();
    std::fs::write(bundle.join("active"), "Work\n").unwrap();
    std::fs::write(bundle.join("packages.json"), "{\"packages\":[]}").unwrap();
    std::fs::create_dir_all(&cfg).unwrap();
    std::fs::write(cfg.join("something"), "keep me").unwrap();

    let reg = data.join("registry.json");
    let refused = restore_bundle(&bundle, &cfg, &reg, false, false).await;
    assert!(refused.is_err(), "a non-empty config must be refused");
    // And refused as a REFUSAL: `Error::Other` here exited 1, which README.md's table
    // defines as "Shall could not carry it out", and never fired `on_guard_refusal`.
    assert!(
        matches!(refused, Err(Error::Refused(_))),
        "a deliberate refusal is `Error::Refused` (exit 3), got {refused:?}"
    );

    // Into a clean directory it restores the declarations.
    let clean = tmp.join("clean");
    let report = restore_bundle(&bundle, &clean, &reg, false, false)
        .await
        .unwrap();
    assert!(report.config_files >= 2);
    assert_eq!(
        std::fs::read_to_string(clean.join("modules/tools.txt")).unwrap(),
        "apt:jq\n"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

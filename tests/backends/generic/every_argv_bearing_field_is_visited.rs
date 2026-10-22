use super::*;

/// Every argv-bearing field, not the four somebody remembered. A placeholder that reaches a
/// manager literally is the failure this whole mechanism would otherwise introduce.
#[test]
fn every_argv_bearing_field_is_visited() {
    let mut cfg = row("m");
    let p = || "{setting.k}".to_string();
    cfg.install_args = vec![p()];
    cfg.remove_args = vec![p()];
    cfg.list_args = vec![p()];
    cfg.search_args = vec![p()];
    cfg.upgrade_args = vec![p()];
    cfg.purge_args = Some(vec![p()]);
    cfg.essential_args = Some(vec![p()]);
    cfg.enumerate_args = Some(vec![p()]);
    cfg.update_args = Some(vec![p()]);
    cfg.repo_add_args = Some(vec![p()]);
    cfg.repo_remove_args = Some(vec![p()]);
    cfg.repo_list_args = Some(vec![p()]);
    cfg.upgrade_reinstall_args = Some(vec![p()]);
    cfg.extra_probes = Some(vec![p()]);
    cfg.version_pin = Some(VersionPin::Inline(p()));
    cfg.property_probes = vec![PropertyProbe {
        property: "x".into(),
        args: vec![p()],
        template: p(),
    }];
    cfg.manual = ManualListing::Command {
        binary: None,
        args: vec![p()],
        format: ManualFormat::BareNames,
    };
    cfg.clean_cache = Some(CacheClean {
        binary: None,
        args: vec![p()],
    });
    cfg.orphan_dry_run = Some(OrphanDryRun {
        binary: None,
        args: vec![p()],
        removes_line_prefix: String::new(),
    });

    cfg.resolve_settings(Some(&settings(&[("k", "V")])))
        .expect("every field resolves");
    let rendered = format!("{cfg:?}");
    assert!(
        !rendered.contains("{setting."),
        "a placeholder survived: {rendered}"
    );
}

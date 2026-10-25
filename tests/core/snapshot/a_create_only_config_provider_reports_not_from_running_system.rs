use super::*;

#[test]
fn a_create_only_config_provider_reports_not_from_running_system() {
    let d = def("lvm");
    let cap = ConfigSnapshotProvider {
        executor: CommandExecutor::new(true, false),
        def: d,
    }
    .restore_capability();
    assert!(!cap.is_live());
    match cap {
        RestoreCapability::NotFromRunningSystem { how } => assert!(!how.is_empty()),
        _ => panic!("a create-only provider must not report Live"),
    }
}

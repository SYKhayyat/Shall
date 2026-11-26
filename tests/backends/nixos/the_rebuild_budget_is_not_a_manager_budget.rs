use super::*;

/// **The number this pins is a fix, not a preference.** `nixos-rebuild switch` evaluates nixpkgs,
/// builds a system closure and activates it; at the five-minute manager default it is cancelled
/// part-way, which every layer downstream reads as a finished command (`#225`).
#[test]
fn a_nixos_rebuild_declares_a_budget_a_rebuild_can_fit_in() {
    let (core, _vfs, _mock) = dry_core();
    let floor = core
        .command_budget()
        .expect("nixos: has to declare a floor, or its rebuild keeps the manager budget");
    assert!(
        floor > std::time::Duration::from_secs(300),
        "the manager default is five minutes and a rebuild does not fit in it: {floor:?}"
    );
}

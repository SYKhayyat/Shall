use super::*;

#[test]
fn shall_data_is_never_inside_your_repo() {
    // II.1. The registry is what Shall OWNS; the modules are what you WANT. When those
    // two can be pointed at each other, owned-but-unwished reads as drift and drift
    // gets removed — which is Monday's bug (V.1).
    let l = layout();
    assert!(!l.registry_file().starts_with(l.config_root()));
    assert!(!l.snapshots_dir().starts_with(l.config_root()));
}

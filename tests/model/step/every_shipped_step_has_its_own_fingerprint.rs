use super::*;

/// Two steps with the same command would share a ledger row and one would report the
/// other's run count.
#[test]
fn every_shipped_step_has_its_own_fingerprint() {
    let rows = rows();
    let mut seen = std::collections::HashMap::new();
    for step in &rows {
        if let Some(other) = seen.insert(fingerprint(step), step.name.clone()) {
            panic!("`{}` and `{}` hash to one ledger row", step.name, other);
        }
    }
}

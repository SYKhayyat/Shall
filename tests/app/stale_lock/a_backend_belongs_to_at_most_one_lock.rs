use super::*;

/// No backend may be claimed by two rows: the second would never be consulted, and which
/// one wins would be the table's declaration order rather than anybody's decision.
#[test]
fn a_backend_belongs_to_at_most_one_lock() {
    let mut seen: Vec<&str> = Vec::new();
    for lock in MANAGER_LOCKS {
        for b in lock.backends {
            assert!(!seen.contains(b), "{b} is claimed by two locks");
            seen.push(b);
        }
    }
}

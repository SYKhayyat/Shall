use super::*;

/// `exec:` is a verb: it must never be keyed into the extras teardown ledger, or a script
/// whose `when` went false would be "undone" (XIII.3's flapping bug).
#[test]
fn an_exec_is_not_an_extra_with_a_teardown_key() {
    let stmt = pv("exec:./bin/enroll-tpm.sh").unwrap();
    assert_eq!(shall::core::extra_key(&stmt).unwrap(), None);
}

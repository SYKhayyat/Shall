use super::*;

/// The SID reads back as a SID, and not as its first character.
///
/// The control for #101 and #106 in one assertion. A null `peUse` out-parameter made this
/// lookup kill the process instead of answering, and `CStr::from_ptr` over
/// `ConvertSidToStringSidW`'s **wide** buffer stopped at the high half of the first `u16`,
/// so the identity of every account on Windows read back as `"S"` — a string that was
/// wrong rather than absent, which is why it flowed all the way to `ProfileList` and out the
/// other side as "no home directory for this account".
#[cfg(windows)]
#[test]
fn the_current_accounts_sid_reads_back_as_a_sid() {
    let account = Account::current().unwrap();
    let sid = account.sid().expect("a Windows account has a SID");
    assert!(
        sid.starts_with("S-1-"),
        "the SID read back as {sid:?}, which is not the identity of {}",
        account.name()
    );
}

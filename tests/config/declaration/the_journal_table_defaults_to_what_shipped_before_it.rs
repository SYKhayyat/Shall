use super::*;

/// The clamp that keeps the buffer bounded lives on the journal, not here — this only
/// pins the shipped number, and that the parent actually carries it. A `Default` on a
/// table the parent does not use is a default nothing reads.
#[test]
fn the_journal_table_defaults_to_what_shipped_before_it() {
    assert_eq!(JournalSettings::default().flush_every, 32);
    assert_eq!(Config::default().journal.flush_every, 32);
}

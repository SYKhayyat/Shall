use super::*;

#[test]
fn what_it_writes_is_what_the_resolver_reads_back() {
    // The two halves of the same fact: if these ever disagree, a `--temp` install is
    // either permanent or already expired.
    let written = absolute_after(now(), "2h").unwrap();
    let read = parse_absolute(&written).expect("must parse back");
    assert_eq!(read, parse_absolute("2026-07-17T14:00").unwrap());
}

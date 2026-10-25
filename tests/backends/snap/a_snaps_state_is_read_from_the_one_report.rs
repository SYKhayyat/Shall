use super::*;

/// Real `snap info` shape. Presence is the `installed:` line — `snap info` answers just as
/// happily for a snap that only exists in the store, and reading that as installed would
/// send every first install down the refresh path.
#[test]
fn a_snaps_state_is_read_from_the_one_report() {
    let classic = "name:      code\n\
                   summary:   Code editing. Redefined.\n\
                   tracking:     latest/stable\n\
                   installed:          1.85.1                     (139) 351MB classic\n";
    assert_eq!(
        installed_state(classic),
        Some(SnapState {
            channel: Some("stable".into()),
            classic: true
        })
    );

    // No notes: the line ends on its size, and a size can never spell `classic`.
    let strict = "name:      hello\n\
                  tracking:     latest/edge\n\
                  installed:          2.10                       (42) 98kB -\n";
    assert_eq!(
        installed_state(strict),
        Some(SnapState {
            channel: Some("edge".into()),
            classic: false
        })
    );

    // Several notes at once, comma-separated — `classic` is one of them and must be found
    // among them rather than by the field being equal to it.
    let both = "installed:  1.0 (7) 2MB classic,disabled\n";
    assert!(installed_state(both).unwrap().classic);

    // In the store, not on the machine.
    assert_eq!(installed_state("name: code\nsummary: x\n"), None);
}

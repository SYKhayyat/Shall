use super::*;

/// The same, for launchd, which was excused from it on the shape of the parser.
///
/// The excuse — *"every other init here puts the name between two positionals"* — was simply
/// untrue of launchd, whose four rows all end in `{name}`. Nobody had run `launchctl` until
/// the terminator probe did, and it disagreed with the row in `core/argv.rs` by name (nightly
/// run 31458415385). Asserted per verb rather than "no row is missing one", because the two
/// that take a flag put it in a different place from the two that do not.
#[test]
fn launchd_ends_its_options_before_the_service() {
    let ld = shipped("launchd");
    for (action, expected) in [
        (
            ServiceAction::Enable,
            vec!["launchctl", "load", "-w", "--", "nginx"],
        ),
        (
            ServiceAction::Disable,
            vec!["launchctl", "unload", "-w", "--", "nginx"],
        ),
        (
            ServiceAction::Start,
            vec!["launchctl", "start", "--", "nginx"],
        ),
        (
            ServiceAction::Stop,
            vec!["launchctl", "stop", "--", "nginx"],
        ),
    ] {
        let planned: Vec<Vec<String>> = ld
            .plan(action, "nginx")
            .into_iter()
            .map(|(_, c)| c)
            .collect();
        assert_eq!(
            planned,
            vec![expected
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()],
            "{action:?}"
        );
    }
}

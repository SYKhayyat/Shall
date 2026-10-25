use super::*;

#[test]
fn a_suspension_takes_the_game_away_until_the_weekend() {
    // V.37, end to end: `absent:` with a date beats the module that wants it, until
    // the date passes — then the module wins again and it comes back.
    let suspended = Declared {
        options: {
            let mut o = Options::default();
            o.insert("until".to_string(), "2026-07-18T00:00");
            o
        },
        origin: Origin::new("modules/focus.txt", 1),
        present: false,
    };
    let wanted = decl("modules/gaming.txt", 5, &[]);

    let during = reconcile("apt:steam", suspended.clone(), wanted.clone(), now()).unwrap();
    assert!(!during.present, "the game is away during the week");

    let after = parse_absolute("2026-07-19T12:00").unwrap();
    let weekend = reconcile("apt:steam", suspended, wanted, after).unwrap();
    assert!(weekend.present, "and comes back at the weekend");
}

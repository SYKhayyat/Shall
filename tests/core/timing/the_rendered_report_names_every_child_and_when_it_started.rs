use super::*;

/// The half [`record`] cannot cover: that [`end`] measures a real elapsed interval.
///
/// One span, one loose lower bound, nothing compared against another span — so there is no
/// ordering for a busy scheduler to invert. A 30 ms sleep that records less than 10 ms is
/// not a slow machine, it is a broken clock.
///
/// **Last, and inside the same test.** The recorder is process-global; a second `#[test]`
/// would run on another thread and add a row to the table the assertions above are
/// counting.
/// **The rendered report names every child, which is the whole of what `#107` was missing.**
///
/// The per-child table has always been printed; `report` threw it into stderr where the
/// fan-out gate captured the process output and then quoted only the summary line, so a
/// failing run named a count and an overlap ratio and not one manager. That is why the issue
/// could say "which seven commands these are is the entire question" and never answer it: the
/// answer was in the string it had in hand and was discarded.
///
/// Asserted on every label, and on the start offsets, because the offsets are what turn "seven
/// children" into "these seven, asked in this order, at these times" — the difference between a
/// count and a diagnosis.
#[test]
fn the_rendered_report_names_every_child_and_when_it_started() {
    let ms = Duration::from_millis;
    let rows = vec![
        Row {
            label: "solus".to_string(),
            calls: 1,
            total: ms(300),
            longest: ms(300),
            first_at: ms(0),
            last_end: ms(300),
        },
        Row {
            label: "winget".to_string(),
            calls: 2,
            total: ms(900),
            longest: ms(500),
            first_at: ms(10),
            last_end: ms(910),
        },
    ];
    let text = render(&rows, ms(920), ms(1200), 7, 2);

    // The summary line stays exactly as it was: the gate parses `child command(s)`,
    // `x overlap` and ` wave(s)` out of it, and a report that answers the diagnosis by
    // breaking its own parser would trade one unreadable failure for another. `wide` is
    // appended after all three for the same reason the rest of this function exists.
    assert!(
        text.contains("3 child command(s) summing to 1.20s"),
        "the aggregate line is what the gate parses; it changed:\n{text}"
    );
    assert!(
        text.contains("2 wide"),
        "the summary line must report how many children shared the busiest instant, which \
         is the one reading a host that starts processes slowly cannot corrupt:\n{text}"
    );
    for label in ["solus", "winget"] {
        assert!(
            text.contains(label),
            "`{label}` was asked and the report does not say so, which is the `#107` \
             failure this test exists to prevent:\n{text}"
        );
    }
    // The start offsets, not just the names: three children in one wave and seven waves is
    // the serialisation itself.
    assert!(
        text.contains("0.00s") && text.contains("0.01s"),
        "the table reports when each child started, which is how a serial run is told from an \
         overlapped one:\n{text}"
    );
    // A label called more than once says so, or a repeated question reads as one.
    assert!(
        text.contains("(2x, longest 0.50s"),
        "a label asked twice reports both calls, since `sbom` asking two questions of one \
         manager is not the same as asking one:\n{text}"
    );

    // **Seven child commands, five programs — the exact shape `#107` misread as seven
    // managers.** The two `npm` rows and the two `pipx` rows are one program each, so the
    // report has to say so: `Shape::wave_ceiling` is calibrated on managers, and the gate was
    // handing it a child count and calling it a manager count.
    let programs = distinct_programs(&rows);
    assert_eq!(calls_of(&rows), 3, "three child commands in this fixture");
    assert_eq!(
        programs, 2,
        "`solus` and `winget` are two programs however often each is asked; a program asked \
         twice is still one program"
    );

    // And the empty run still says it asked nobody, rather than rendering an empty table that
    // reads like a truncated one.
    let empty = render(&[], ms(50), Duration::ZERO, 0, 0);
    assert!(
        empty.contains("no child commands"),
        "an empty run must say so in words:\n{empty}"
    );
    assert!(
        !empty.contains("command\n"),
        "an empty run must not print a table header, which would look like rows lost:\n{empty}"
    );
}

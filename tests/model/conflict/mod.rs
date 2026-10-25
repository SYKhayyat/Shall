pub use chrono::{DateTime, Utc};
pub use shall::config::grammar::{GrammarError, Options, Origin};
pub use shall::model::conflict::*;
pub use shall::model::dated::{dating_of, parse_absolute, Dating};

fn now() -> DateTime<Utc> {
    parse_absolute("2026-07-16T12:00").unwrap()
}
fn decl(file: &str, line: usize, pairs: &[(&str, &str)]) -> Declared {
    let mut options = Options::default();
    for (k, v) in pairs {
        options.insert(*k, *v);
    }
    Declared {
        options,
        origin: Origin::new(file, line),
        present: true,
    }
}
fn absent(file: &str, line: usize) -> Declared {
    Declared {
        options: Options::default(),
        origin: Origin::new(file, line),
        present: false,
    }
}
mod a_counting_dated_line_beats_an_undated_one;
mod a_lapsed_line_stops_counting_and_the_undated_one_wins;
mod a_suspension_takes_the_game_away_until_the_weekend;
mod declaring_the_same_thing_twice_is_not_a_disagreement;
mod options_only_one_side_sets_are_merged_not_fought_over;
mod present_and_absent_contradict;
mod the_error_names_both_files_and_lines;
mod two_contradicting_declarations_are_an_error;
mod two_lapsed_lines_do_not_error_against_each_other;

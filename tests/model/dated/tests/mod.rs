pub use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
pub use shall::config::grammar::Options;
pub use shall::model::dated::*;

fn at(s: &str) -> DateTime<Utc> {
    parse_absolute(s).unwrap()
}
fn opts(pairs: &[(&str, &str)]) -> Options {
    let mut o = Options::default();
    for (k, v) in pairs {
        o.insert(*k, *v);
    }
    o
}
mod a_dated_line_counts_until_its_date;
mod a_dated_line_stops_counting_once_its_date_passes;
mod an_undated_line_counts_forever;
mod every_absolute_form_the_grammar_accepts_is_readable_here;
mod the_moment_itself_has_passed;
mod until_is_the_mirror_of_expires;

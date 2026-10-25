pub use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
pub use shall::config::grammar::Options;
pub use shall::model::dated::*;

fn now() -> DateTime<Utc> {
    parse_absolute("2026-07-17T12:00").unwrap()
}
mod a_duration_becomes_the_moment_it_runs_out;
mod nonsense_is_refused_rather_than_guessed;
mod what_it_writes_is_what_the_resolver_reads_back;

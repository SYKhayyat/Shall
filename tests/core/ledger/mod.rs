pub use shall::core::ledger::*;
pub use shall::core::{Error, Result};
use std::collections::BTreeMap;
use std::path::Path;
use tempfile::TempDir;

#[derive(Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Probe {
    #[serde(default)]
    entries: BTreeMap<String, String>,
}
impl LockFile for Probe {
    const WHAT: &'static str = "the probe ledger";
}
mod a_corrupt_file_is_an_error_and_not_an_empty_ledger;
mod a_corrupt_json_record_is_an_error_and_a_missing_one_is_empty;
mod a_missing_file_loads_empty_rather_than_failing;
mod it_round_trips_through_the_file_and_creates_the_directory;

mod ledger_file_rules_tests;

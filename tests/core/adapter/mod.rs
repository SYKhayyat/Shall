pub use shall::core::adapter::*;

struct Row {
    name: String,
    os: Option<String>,
    broken: Option<&'static str>,
    live: Option<String>,
}
impl AdapterRow for Row {
    const WHAT: &'static str = "test adapter";
    fn name(&self) -> &str {
        &self.name
    }
    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }
    fn why_unusable(&self) -> Option<&'static str> {
        self.broken
    }
}
impl Detected for Row {
    fn detect_command(&self) -> &str {
        &self.name
    }
    fn detect_file(&self) -> Option<&str> {
        self.live.as_deref()
    }
}
fn row(name: &str, os: Option<&str>) -> Row {
    Row {
        name: name.into(),
        os: os.map(str::to_string),
        broken: None,
        live: None,
    }
}
mod a_filled_template_splits_into_a_program_and_its_arguments;
mod a_row_naming_an_os_applies_only_there;
mod an_unusable_row_never_reaches_the_merged_table;
mod placeholders_are_filled_left_to_right;
mod the_first_claim_on_a_name_wins_and_a_second_is_dropped;
mod the_first_row_this_machine_has_is_the_one_chosen;
mod the_floor_drops_a_nameless_row_and_says_which_field;
mod usable_keeps_two_rows_that_share_a_name;

mod adapter_tables_share_one_mechanism_tests;

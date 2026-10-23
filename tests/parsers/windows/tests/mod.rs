pub use shall::core::Package;
pub use shall::parsers::windows::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

fn scoop_list_fixture() -> String {
    let row = |name: &str, ver: &str, src: &str, updated: &str, info: &str| {
        format!("{:<9}{:<16}{:<7}{:<20}{}", name, ver, src, updated, info)
    };
    [
        "Installed apps:".to_string(),
        String::new(),
        row("Name", "Version", "Source", "Updated", "Info"),
        row("----", "-------", "------", "-------", "----"),
        row("7zip", "26.00", "main", "2026-04-19 07:09:55", ""),
        row("jq", "", "", "2026-07-21 13:48:29", "Install failed"),
        row("ripgrep", "15.1.0", "main", "2026-07-08 15:38:44", ""),
    ]
    .join("\n")
}
fn wrow(name: &str, id: &str, ver: &str, avail: &str, src: &str) -> String {
    format!("{:<24}{:<40}{:<14}{:<14}{}", name, id, ver, avail, src)
}
fn winget_list_fixture() -> String {
    include_str!("../../../../tests/fixtures/winget/list.txt").to_string()
}
mod scoop_list_drops_a_failed_install;
mod scoop_list_ingests_no_header_or_separator;
mod scoop_list_reads_versions_from_the_version_column;
mod scoop_search_parses_modern_table;
mod scoop_search_survives_empty_trailing_columns;
mod test_choco_list_parsing;
mod winget_list_handles_cr_spinner_header;
mod winget_list_uses_columns_not_whitespace;
mod winget_search_parses_columns;

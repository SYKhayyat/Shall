pub use shall::model::introduced::*;

const LOG: &str = "\
c3333333333333333333333333333333333333333\t2026-05-02\tpin ripgrep to 14.1
b2222222222222222222222222222222222222222\t2026-04-10\tmove ripgrep into tools
a1111111111111111111111111111111111111111\t2026-03-14\tadd ripgrep
";
mod a_commit_with_an_empty_subject_is_not_a_parse_failure;
mod a_line_that_was_only_ever_added_once_still_works;
mod a_malformed_line_yields_nothing;
mod a_tab_in_the_subject_does_not_shift_the_fields;
mod no_history_is_no_answer;
mod the_date_format_is_pinned;
mod the_oldest_commit_is_the_one_that_introduced_it;
mod the_query_uses_the_pickaxe_and_is_limited_to_the_given_paths;
mod the_summary_reads_as_one_line;

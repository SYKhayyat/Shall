pub use shall::core::Package;
pub use shall::parsers::*;

const COMPOSER_WITH_BANNER: &str = concat!(
    "Changed current directory to /root/.composer\n",
    r#"{"installed":[{"name":"psr/log","version":"1.1.4"}]}"#,
    "\n"
);
mod a_banner_above_the_document_does_not_hide_it;
mod a_brace_in_the_banner_does_not_get_read_as_the_document;
mod a_note_below_the_document_does_not_hide_it_either;
mod bare_names_handles_apk_world_entries;
mod bare_names_parses_apt_mark_showmanual;
mod bare_names_strips_the_architecture_qualifier;
mod output_with_no_document_is_none_and_not_null;
mod the_container_says_which_of_the_three_answers_this_is;

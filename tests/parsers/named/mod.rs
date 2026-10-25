pub use shall::core::Package;
pub use shall::parsers::named::*;
pub use shall::parsers::{
    apt, bsd, common, conda, dnf, dotnet, ecosystem, language, macos, pacman, pkgsrc, windows,
    OutputParser, ParseResult, Unrecognised,
};
mod a_manager_with_no_listing_verb_says_so_rather_than_reporting_nothing;
mod a_name_nothing_answers_to_is_none;
mod a_row_with_no_search_reader_answers_empty_rather_than_panicking;
mod every_listed_reader_resolves;
mod the_backend_reaches_the_reader;

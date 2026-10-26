//! Mirrors for parser modules whose inline tests have moved out of `src/`.

mod apt;
mod bsd;
mod common;
mod conda;
mod dnf;
mod dotnet;
mod ecosystem;
mod language;
mod macos;
mod named;
mod pacman;
mod pkgsrc;
mod root;
mod utils;
mod windows;

mod a_parser_can_say_it_did_not_understand_tests;
mod installed_listing_fixture_tests;
mod parser_fixture_tests;

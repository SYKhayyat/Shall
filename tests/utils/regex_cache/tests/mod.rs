use once_cell::sync::Lazy;
use regex::Regex;
pub use shall::utils::regex_cache::*;
use std::sync::Arc;
mod a_cached_regex_still_matches;
mod a_pattern_that_will_not_compile_says_so_every_time;
mod the_same_pattern_is_compiled_once;

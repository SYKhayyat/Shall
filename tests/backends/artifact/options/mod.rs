pub use shall::backends::artifact::format::FormatOrder;
pub use shall::backends::artifact::options::*;
pub use shall::backends::artifact::pattern::AssetPattern;
pub use shall::backends::artifact::Format;
pub use shall::config::grammar::Options;

fn opts(pairs: &[(&str, &str)]) -> Options {
    let mut o = Options::default();
    for (k, v) in pairs {
        for part in v.split(';') {
            o.insert(*k, part);
        }
    }
    o
}
mod a_line_replaces_the_fallback_rather_than_extending_it;
mod a_repeated_key_keeps_the_order_it_was_written_in;
mod a_single_format_needs_no_separator;
mod an_asset_pattern_is_compiled_once_here;
mod an_empty_bin_is_the_same_as_no_bin;
mod an_unknown_format_is_refused_here_too;
mod download_only_defaults_off_and_reads_a_bare_flag;
mod no_formats_option_means_the_default_applies;

use super::*;

/// **The family, not the finding.** `helm plugin list` with no plugins prints its header
/// and stops, and every reader in this module dropped the header from the packages while
/// leaving it in the candidates — so `or_unrecognised` saw one data line yielding nothing
/// and refused the whole listing. A manager with nothing installed is not a manager nobody
/// can read, and the two answers send the planner in opposite directions.
///
/// One case per reader that takes a header, because the fix is in `is_noise_line` and a
/// reader that stops calling it would go back to refusing.
#[test]
fn a_listing_that_is_only_a_header_is_empty_rather_than_unreadable() {
    /// A backend, the reader its row names, and a listing of nothing but a header.
    type HeaderCase = (&'static str, fn(&str, &str) -> ParseResult, &'static str);
    let cases: [HeaderCase; 6] = [
        ("helm", ws_name_version, "NAME\tVERSION\tDESCRIPTION\n"),
        ("krew", names_only, "PLUGIN\n"),
        ("cabal", cabal_list, "Package Version\n"),
        ("uv", uv_tool_list, "NAME VERSION\n"),
        ("nimble", nimble_list, "Name Version\n"),
        ("eopkg", eopkg_list, "Package - Description\n"),
    ];
    for (backend, reader, header) in cases {
        let pkgs = reader(header, backend)
            .unwrap_or_else(|e| panic!("{backend}: a bare header was refused — {e:?}"));
        assert!(
            pkgs.is_empty(),
            "{backend}: read {pkgs:?} out of a header row"
        );
    }
}

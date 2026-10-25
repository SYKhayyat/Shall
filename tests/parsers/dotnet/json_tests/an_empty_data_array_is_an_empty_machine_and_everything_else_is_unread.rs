use super::*;

/// An SDK too old for `--format json` fails rather than printing the table, so this never
/// sees one — but if the negotiation ever regressed, reading a table as JSON must report
/// nothing rather than inventing a package from a header row.
#[test]
fn an_empty_data_array_is_an_empty_machine_and_everything_else_is_unread() {
    assert!(parse_dotnet_list_json(r#"{"version":1,"data":[]}"#)
        .expect("dotnet with no global tools returns an empty `data` array")
        .is_empty());
    // The table form is what an SDK *without* `--format json` prints, and the empty string
    // is what it prints when the command fails outright. Both used to arrive as "no global
    // tools are installed" — for exactly the users on older tooling, who are the least
    // likely to notice. `{"version":1}` with no `data` is the third: a schema this does not
    // read.
    for broken in [
        "Package Id  Version\n----\nx  1.0\n",
        "",
        r#"{"version":1}"#,
    ] {
        let err = parse_dotnet_list_json(broken)
            .expect_err("output this reader could not read is not an empty machine");
        assert_eq!(err.backend, "dotnet", "{broken}");
    }
}

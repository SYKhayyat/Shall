use super::*;
use shall::core::adapter::AdapterRow;

/// A row whose `reads` names no reader is a row that answers with silence — the same defect
/// `every_row_can_read_what_it_asks_for` refuses for a listing, and the reason the two columns
/// are checked against each other rather than each on its own.
#[test]
fn the_shipped_rows_parse_and_every_reader_answers() {
    let rows = owners(Vec::new());
    assert!(
        rows.len() >= 5,
        "the shipped table names the managers that can answer: {rows:?}"
    );
    for row in &rows {
        assert!(
            row.unusable().is_none(),
            "{} is shipped and unusable: {:?}",
            row.name,
            row.unusable()
        );
        assert!(
            shall::parsers::named::owning(&row.reads).is_some(),
            "{} names the reader `{}`, which nothing answers to",
            row.name,
            row.reads
        );
        let owns = format!("{row:?}");
        assert!(
            owns.contains("{path}"),
            "{}'s `owns` argv has no {{path}} to substitute, so the question is never asked",
            row.name
        );
    }
}

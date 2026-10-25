use super::*;

#[test]
fn an_unusable_row_never_reaches_the_merged_table() {
    let merged = merge(vec![
        Row {
            name: "broken".into(),
            os: None,
            broken: Some("it cannot list its rules"),
            live: None,
        },
        row("ufw", None),
    ]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].name, "ufw");
}

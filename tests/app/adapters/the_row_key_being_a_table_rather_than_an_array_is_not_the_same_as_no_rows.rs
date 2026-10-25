use super::*;

#[test]
fn the_row_key_being_a_table_rather_than_an_array_is_not_the_same_as_no_rows() {
    // `[backend]` instead of `[[backend]]` — one bracket, and every reader sees nothing.
    // Reported as malformed rather than empty, because the user plainly meant a row.
    let s = standing(&[("backends", "[backend]\nname = \"a\"\n")], "backends");
    assert!(matches!(s, Standing::Malformed(_)), "{s:?}");
    assert!(
        s.detail().is_some_and(|d| d.contains("[[backend]]")),
        "the message must show what a row looks like: {s:?}"
    );
}

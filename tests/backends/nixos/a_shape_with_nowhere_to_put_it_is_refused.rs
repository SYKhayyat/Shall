use super::*;

/// A file this cannot place the import in is refused, not guessed at. Writing something that
/// does not parse into somebody's boot configuration is the one outcome worth failing over.
#[test]
fn a_shape_with_nowhere_to_put_it_is_refused() {
    assert_eq!(with_import("", GENERATED), None);
    assert_eq!(
        with_import(
            "# just a comment
",
            GENERATED
        ),
        None
    );
}

use super::*;

/// K17: a store Shall has never heard of is a row, not a release. This is the whole point
/// of the ruling — the adapter below is driven from a definition, through the same code
/// path the shipped one uses.
#[test]
fn a_store_with_no_compiled_in_support_is_driven_from_a_row() {
    let all = adapters(vec![row("kde", "kwriteconfig6")]);
    let kde = all
        .iter()
        .find(|a| a.name == "kde")
        .expect("the user row loaded");
    let (prog, args) = kde.write_command(
        Scope::User,
        "kdeglobals/General",
        "ColorScheme",
        "BreezeDark",
    );
    assert_eq!(prog, "kwriteconfig6");
    assert_eq!(
        args,
        vec!["write", "kdeglobals/General", "ColorScheme", "BreezeDark"]
    );
}

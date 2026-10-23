use super::*;

#[test]
fn subcommand_index_skips_flags_and_their_values() {
    assert_eq!(find_subcommand_index(&argv(&["shall", "up"])), Some(1));
    assert_eq!(
        find_subcommand_index(&argv(&["shall", "-c", "x", "up"])),
        Some(3)
    );
    assert_eq!(
        find_subcommand_index(&argv(&["shall", "--dry-run", "up"])),
        Some(2)
    );
    assert_eq!(find_subcommand_index(&argv(&["shall", "--version"])), None);
}

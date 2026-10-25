use super::*;

/// The other two ways the question is unanswerable, each for its own reason and neither of
/// them "the value differs". A machine with no store cannot be asked; a name that is not
/// `SCHEMA/KEY` addresses nothing to ask about.
#[tokio::test]
async fn a_machine_with_no_store_and_a_name_with_no_key_are_both_unanswerable() {
    let none = SettingBackendCore::new(CommandExecutor::new(false, false), vec![]);
    assert_eq!(none.holds("org.gnome.x/k", "v", None, None).await, None);

    let core = store_reading("prefer-dark");
    assert_eq!(core.holds("dark", "prefer-dark", None, None).await, None);
}

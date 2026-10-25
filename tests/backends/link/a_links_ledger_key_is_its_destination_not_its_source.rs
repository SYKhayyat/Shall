//! The defect found while building T6: the teardown key named the declaration's SOURCE, so an
//! undo deleted the file in the user's dotfiles repo and left the deployed copy in place —
//! exactly backwards. The key is the destination now, and this is the assertion that says so.

use super::*;

#[test]
fn a_links_ledger_key_is_its_destination_not_its_source() {
    use shall::config::grammar::{Options, Statement};
    use shall::core::extras_lock::extra_key;

    let mut opts = Options::default();
    opts.insert("target".to_string(), "~/.gitconfig");
    let key = extra_key(&Statement::Link("dotfiles/gitconfig".into(), opts))
        .unwrap()
        .unwrap();

    let want = resolve_target("~/.gitconfig").unwrap();
    assert_eq!(key.kind, shall::config::grammar::ResourceKind::Link);
    assert_eq!(key.subject, want.display().to_string());
    assert_eq!(key.to_string(), format!("link:{}", want.display()));
    assert!(
        !key.subject.contains("dotfiles/gitconfig"),
        "the undo would be handed the source file: {}",
        key
    );
    // And the round trip, which is the property the ledger depends on: what is written is what
    // is read back.
    assert_eq!(
        key.to_string()
            .parse::<shall::core::extras_lock::ExtraKey>(),
        Ok(key)
    );
}

use super::*;

/// The freeze list narrows what a bare `lock` does, in the same words the command takes —
/// a group, a kind, or `everything` minus an exclusion. All three spellings, because a
/// vocabulary that works on the command line and not in the file is two vocabularies.
#[test]
fn the_freeze_list_narrows_a_bare_lock_in_the_commands_own_words() {
    use shall::core::lock_kind::LockKind;

    let group = LockSettings {
        freeze: vec!["scripts".into()],
        ..LockSettings::default()
    };
    assert!(group.freezes().contains(&LockKind::Exec));
    assert!(!group.freezes().contains(&LockKind::Versions));

    let one_kind = LockSettings {
        freeze: vec!["exec".into()],
        ..LockSettings::default()
    };
    assert_eq!(one_kind.freezes(), vec![LockKind::Exec]);

    let subtracted = LockSettings {
        except: vec!["versions".into()],
        ..LockSettings::default()
    };
    assert!(!subtracted.freezes().contains(&LockKind::Versions));
    assert!(subtracted.freezes().contains(&LockKind::Backends));
    assert_eq!(
        subtracted.freezes().len(),
        shall::core::lock_kind::ALL.len() - 1
    );
}

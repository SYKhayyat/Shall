use super::*;

/// **A `[lock]` section that will not parse falls back to everything rather than refusing
/// the command.** A typo in a preference that narrows a default must not be the thing that
/// stops a machine approving a script.
#[test]
fn an_unparseable_freeze_list_falls_back_to_everything() {
    let nonsense = LockSettings {
        freeze: vec!["firewall".into()],
        ..LockSettings::default()
    };
    assert_eq!(nonsense.freezes(), shall::core::lock_kind::ALL.to_vec());

    let empty = LockSettings {
        freeze: vec!["everything".into()],
        except: vec!["everything".into()],
        ..LockSettings::default()
    };
    assert_eq!(empty.freezes(), shall::core::lock_kind::ALL.to_vec());
}

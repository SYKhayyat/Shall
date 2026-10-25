use super::*;

/// These inits take the service between two positionals, so there is nowhere a `--` could go
/// — and each of them would read it as the service name.
///
/// **`launchd` was on this list by assumption and has been taken off it.** Its four rows all
/// end in `{name}`, so a terminator has somewhere to go after all, and the differential probe
/// measured launchctl honouring one on macos-latest: `load -w -- <x>`, `unload -w -- <x>`,
/// `start -- <x>` and `stop -- <x>` are each identical to the same line without it, in exit
/// code, in output, and in how the operand is echoed (nightly run 31458415385, 2026-08-11).
/// The row in `core/argv.rs` says the same thing; these are the two layers that have to agree.
#[test]
fn the_other_inits_deliberately_emit_no_terminator() {
    for name in ["openrc", "sysvinit", "windows-sc"] {
        let p = shipped(name);
        for action in [
            ServiceAction::Enable,
            ServiceAction::Disable,
            ServiceAction::Start,
            ServiceAction::Stop,
            ServiceAction::Restart,
        ] {
            for (_, cmd) in p.plan(action, "nginx") {
                assert!(
                    !cmd.iter().any(|a| a == "--"),
                    "{}/{:?} emitted a terminator",
                    name,
                    action,
                );
            }
        }
    }
}

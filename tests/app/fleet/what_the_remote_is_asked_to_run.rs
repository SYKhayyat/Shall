use super::*;

/// The commands sent over the wire are commands. `shall check --json` sat here for as long
/// as `fleet` existed; `tests/named_commands_exist_tests.rs` now reads these two constants
/// out of the source and checks them against clap, and this says the same thing from
/// inside, where a reader of this module can see it.
#[test]
fn what_the_remote_is_asked_to_run() {
    for cmd in [REMOTE_CHECK, REMOTE_SYNC] {
        let verb = cmd.split_whitespace().nth(1).expect("a verb");
        assert!(
            shall::cli::args::Cli::command()
                .get_subcommands()
                .any(|s| s.get_name() == verb),
            "`{}` is not a command Shall has, so no host can answer it",
            cmd
        );
    }
}

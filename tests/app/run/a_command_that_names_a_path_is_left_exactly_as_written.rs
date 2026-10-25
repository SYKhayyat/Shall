use super::*;

/// A command that is already a path is not a `PATH` question. `shall run ./build.sh` names
/// one file, and re-resolving it through directories would run a different one.
#[tokio::test]
async fn a_command_that_names_a_path_is_left_exactly_as_written() {
    for written in ["./build.sh", "/usr/bin/jq", r"C:\tools\jq.exe"] {
        assert_eq!(real_program_on(written, None).await, written);
    }
}

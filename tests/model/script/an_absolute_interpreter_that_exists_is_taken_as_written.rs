use super::*;

/// An absolute interpreter that exists is run as written — on Unix that is exactly the
/// binary the kernel would have chosen, which is what keeps honouring the shebang here from
/// changing anything there.
#[test]
fn an_absolute_interpreter_that_exists_is_taken_as_written() {
    let interpreter = an_interpreter_that_exists();
    let launch = launch_for(
        Path::new("/tmp/hook"),
        &format!("#!{} -u\nbody\n", interpreter),
    )
    .expect("launch");
    assert_eq!(launch.program, interpreter);
    assert_eq!(launch.args, ["-u", "/tmp/hook"]);
}

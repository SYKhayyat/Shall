use super::*;

/// Resolving the interpreter must not eat the flag beside it — `powershell` without `-File`
/// reads the provider as an expression rather than running it as a file.
#[test]
fn resolving_the_interpreter_keeps_the_flags_that_follow_it() {
    for (file, flag) in [
        ("vars.ps1", "-File"),
        ("vars.bat", "/c"),
        ("vars.cmd", "/c"),
    ] {
        let argv = interpreter_argv(Path::new(file));
        assert_eq!(argv.len(), 2, "{}: {:?}", file, argv);
        assert_eq!(argv[1].to_string_lossy(), flag, "{}", file);
    }
}

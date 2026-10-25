use super::*;

#[test]
fn a_script_with_no_shebang_gets_the_platform_default() {
    let launch = launch_for(Path::new("/tmp/hook"), "echo hi\n").expect("launch");
    if cfg!(windows) {
        assert_eq!(launch.program, "powershell");
        assert!(
            launch.args.iter().any(|a| a == "-NoProfile"),
            "{:?}",
            launch
        );
        assert!(launch.args.iter().any(|a| a == "-File"), "{:?}", launch);
    } else {
        assert_eq!(launch.program, "sh");
        assert_eq!(launch.args.len(), 1);
    }
}

use super::*;

#[cfg(unix)]
#[test]
fn escalation_and_account_switching_compose_into_one_wrapper() {
    let account = another_account();
    let env = account_env(&account);
    let name = account.name().to_string();
    let home = account.home().display().to_string();
    let command = [
        "gsettings",
        "get",
        "org.gnome.desktop.interface",
        "gtk-theme",
    ];
    for escalate in [false, true] {
        let (cmd, args) =
            CommandExecutor::account_argv("gsettings", &command, escalate, Some(&account), &env);
        let line = std::iter::once(cmd.as_str())
            .chain(args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(cmd, "sudo", "escalate={escalate}: {line}");
        assert_eq!(
            &args[..5],
            &["-n", "-u", name.as_str(), "--", "env"],
            "escalate={escalate}: {line}"
        );
        for once in ["-n", "-u", "--", "env"] {
            assert_eq!(
                args.iter().filter(|a| *a == once).count(),
                1,
                "`{once}` appears {} times (escalate={escalate}): {line}",
                args.iter().filter(|a| *a == once).count()
            );
        }
        assert!(
            args.contains(&format!("HOME={home}")),
            "the target's home is not in the wrapper (escalate={escalate}): {line}"
        );
        assert!(
            args.contains(&format!("USER={name}")),
            "the target's name is not in the wrapper (escalate={escalate}): {line}"
        );
        assert_eq!(
            args[args.len() - command.len()..],
            command,
            "the command did not survive the wrapper (escalate={escalate}): {line}"
        );
    }
}

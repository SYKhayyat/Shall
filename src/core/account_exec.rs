use crate::core::account::Account;
use crate::core::executor::{CommandExecutor, ReadOutcome, StdOutput};
use crate::core::{Error, Result};
use std::collections::HashMap;

impl CommandExecutor {
    pub async fn run_as(
        &self,
        account: &Account,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<StdOutput> {
        let output = self.run_on_as(account, cmd, args, sudo).await?;
        let checked = self.ensure_status(cmd, output);
        self.forget_run_scoped_answers();
        checked
    }

    pub(crate) async fn run_on_as(
        &self,
        account: &Account,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<StdOutput> {
        self.run_on(&self.inner, cmd, args, sudo, Some(account))
            .await
    }

    pub(crate) async fn read_raw_as(
        &self,
        account: &Account,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<StdOutput> {
        self.run_on(&self.reader, cmd, args, sudo, Some(account))
            .await
    }

    pub(crate) fn account_target<'a>(
        &self,
        cmd: &str,
        as_account: Option<&'a Account>,
    ) -> Result<Option<&'a Account>> {
        let target = as_account.filter(|account| !account.is_this_process());
        if let Some(account) = target {
            if !self.can_become() {
                return Err(Error::Refused(format!(
                    "`{}` acts for the account `{}`, and this Shall cannot act as another \
                     account: the tools that store per-account state read the invoking \
                     account's identity out of the environment and the session bus, so a \
                     command run from here would change this account's settings while the \
                     declaration names `{}`. Re-run from a shell logged in as `{}`, or as root \
                     where the identity can be switched.",
                    cmd,
                    account.name(),
                    account.name(),
                    account.name()
                )));
            }
        }
        Ok(target)
    }

    pub(crate) fn account_argv(
        cmd: &str,
        args: &[&str],
        escalate: bool,
        target: Option<&Account>,
        env: &HashMap<String, String>,
    ) -> (String, Vec<String>) {
        if let Some(account) = target {
            let mut composed = vec![
                "-n".to_string(),
                "-u".to_string(),
                account.name().to_string(),
                "--".to_string(),
                "env".to_string(),
            ];
            let mut pairs: Vec<(&String, &String)> = env.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(b.0));
            composed.extend(
                pairs
                    .into_iter()
                    .map(|(key, value)| format!("{key}={value}")),
            );
            composed.push(cmd.to_string());
            composed.extend(args.iter().map(|a| (*a).to_string()));
            return ("sudo".to_string(), composed);
        }
        if escalate {
            let mut composed = vec!["-n".to_string(), cmd.to_string()];
            composed.extend(args.iter().map(|a| (*a).to_string()));
            return ("sudo".to_string(), composed);
        }
        (
            cmd.to_string(),
            args.iter().map(|a| (*a).to_string()).collect(),
        )
    }

    pub async fn probe_output_as(
        &self,
        account: &Account,
        cmd: &str,
        args: &[&str],
    ) -> Result<String> {
        let output = self.read_raw_as(account, cmd, args, false).await?;
        if !output.status.success() && !self.is_benign_exit(output.status.code()) {
            return Err(self.answerless_read(cmd, args, &output));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }

    pub async fn run_output_as(
        &self,
        account: &Account,
        cmd: &str,
        args: &[&str],
    ) -> Result<String> {
        match self
            .read_with_retry_as(Some(account), cmd, args, false)
            .await?
        {
            ReadOutcome::Output(text) => Ok(text),
            ReadOutcome::Answerless(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod account_tests {
    use super::CommandExecutor;
    use crate::core::account::Account;
    use crate::core::executor::MockExecutor;
    use dashmap::DashMap;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
        let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let e =
            CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        (e, mock)
    }

    #[cfg(unix)]
    fn another_account() -> Account {
        for name in ["nobody", "daemon", "bin", "root"] {
            if let Ok(account) = Account::resolve(name) {
                if !account.is_this_process() {
                    return account;
                }
            }
        }
        panic!("this host names no account other than the invoking one");
    }

    // Only the `cfg(unix)` tests below call it, so on Windows it is dead code — a warning, and
    // this project's gate rates warnings fatal. Gated rather than deleted, because a helper the
    // Unix tests share is a helper the next Unix test should reach for.
    #[cfg(unix)]
    fn account_env(account: &Account) -> HashMap<String, String> {
        account.session_env().into_iter().collect()
    }

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
            let (cmd, args) = CommandExecutor::account_argv(
                "gsettings",
                &command,
                escalate,
                Some(&account),
                &env,
            );
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

    #[cfg(unix)]
    #[test]
    fn the_variables_inside_the_wrapper_are_in_a_stable_order() {
        let account = another_account();
        let env = account_env(&account);
        let (_, args) =
            CommandExecutor::account_argv("gsettings", &["get", "x"], false, Some(&account), &env);
        let pairs = &args[5..args.len() - 3];
        assert!(!pairs.is_empty(), "nothing was carried in: {args:?}");
        let mut sorted = pairs.to_vec();
        sorted.sort();
        assert_eq!(pairs, sorted.as_slice(), "{args:?}");
    }

    #[test]
    fn escalation_without_an_account_is_unchanged() {
        let env = HashMap::new();
        assert_eq!(
            CommandExecutor::account_argv("apt-get", &["install", "-y", "nginx"], true, None, &env),
            (
                "sudo".to_string(),
                vec!["-n", "apt-get", "install", "-y", "nginx"]
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(
            CommandExecutor::account_argv(
                "apt-get",
                &["install", "-y", "nginx"],
                false,
                None,
                &env
            ),
            (
                "apt-get".to_string(),
                vec!["install", "-y", "nginx"]
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>()
            )
        );
    }

    #[tokio::test]
    async fn a_command_for_the_account_this_process_is_runs_unwrapped() {
        let (e, mock) = wired();
        let account = Account::current().expect("this process has an account");
        e.read_raw_as(
            &account,
            "gsettings",
            &["get", "org.gnome.desktop.interface", "gtk-theme"],
            false,
        )
        .await
        .expect("the mock answers");
        assert_eq!(
            mock.get_calls().await,
            vec!["gsettings get org.gnome.desktop.interface gtk-theme".to_string()],
            "the account this process is added a wrapper that was not there before"
        );
        let home = account.home().display().to_string();
        let env = mock.last_env.lock().await.clone();
        assert_eq!(env.get("HOME").map(String::as_str), Some(home.as_str()));
        assert_eq!(env.get("USER").map(String::as_str), Some(account.name()));
    }

    #[cfg(unix)]
    #[test]
    fn the_account_this_process_is_is_not_handed_to_the_composer_as_a_target() {
        let account = Account::current().expect("this process has an account");
        assert!(account.is_this_process());
        assert!(Some(&account)
            .filter(|candidate| !candidate.is_this_process())
            .is_none());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_command_for_another_account_is_refused_where_this_process_cannot_become_it() {
        if CommandExecutor::is_root() {
            return;
        }
        let (e, mock) = wired();
        let account = another_account();
        let err = e
            .read_raw_as(&account, "gsettings", &["get", "x"], false)
            .await
            .expect_err("a refused account must not run the command");
        let message = err.to_string();
        assert!(message.contains(account.name()), "{message}");
        assert!(mock.get_calls().await.is_empty(), "{message}");
    }
}

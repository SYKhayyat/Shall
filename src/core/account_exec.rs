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
        self.forget_run_scoped_answers(None);
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

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub async fn read_raw_as(
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

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn account_argv(
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

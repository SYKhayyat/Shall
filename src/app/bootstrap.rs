//! A script that reproduces a machine without Shall (#66, #67, II.94).
//!
//! **The commands are the ones a sync would run.** They come from
//! [`Installable::planned_installs`](crate::core::manager::Installable::planned_installs), which
//! is the same builder `install` runs through — not a second description of "how `apt` installs a
//! package". A second description is free to disagree with the first the moment a capability flag
//! moves, which is the finding of `#151` and `#162`; a script that disagrees with the tool is
//! worse than no script, because it looks authoritative.
//!
//! This module holds the artifact and its renderers, and no policy. What must not be in the
//! script, and what an export does when it cannot say something, is
//! [`II.94`](docs/spec/target-state.md) and lives in the verb.

/// One line of the artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// A command, argv intact, exactly as a sync would have run it.
    Command { program: String, args: Vec<String> },
}

/// A line the format could not carry, kept so the reader is told rather than left with a hole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Omission {
    /// The declaration, as the config spells it: `file:line key`.
    pub line: String,
    /// Why this artifact cannot say it, in one sentence.
    pub why: String,
}

/// What an export produced, and what it could not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Script {
    pub steps: Vec<Step>,
    pub omitted: Vec<Omission>,
}

impl Script {
    /// Add a command, with escalation placed as the machine needs it.
    ///
    /// **`sudo`, not `sudo -n`.** The product adds `-n` because it primes credentials once per
    /// run (`S88`); a bootstrap script is run by somebody who has not primed anything, and the
    /// first escalated command there should be allowed to ask.
    pub fn command(&mut self, program: &str, args: Vec<String>, sudo: bool) {
        let (program, args) = if sudo {
            let mut prefixed = vec![program.to_string()];
            prefixed.extend(args);
            ("sudo".to_string(), prefixed)
        } else {
            (program.to_string(), args)
        };
        self.steps.push(Step::Command { program, args });
    }

    /// Whether there is anything to run at all.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The artifact, in the format asked for.
    pub fn render(&self, format: Format, header: &str) -> String {
        match format {
            Format::Bash => self.to_bash(header),
        }
    }

    fn to_bash(&self, header: &str) -> String {
        let mut out = String::from("#!/usr/bin/env bash\n");
        out.push_str(header);
        // No `-u`: a step may legitimately mention a variable the machine has not set, and
        // stopping halfway through a bootstrap leaves a machine in a state nobody planned.
        out.push_str("set -eo pipefail\n\n");
        for step in &self.steps {
            let Step::Command { program, args } = step;
            out.push_str(&shell_line(program, args));
            out.push('\n');
        }
        if !self.omitted.is_empty() {
            out.push_str("\n# What this script does NOT do, and why:\n");
            for omission in &self.omitted {
                out.push_str(&format!("#   {} — {}\n", omission.line, omission.why));
            }
        }
        out
    }
}

fn shell_line(program: &str, args: &[String]) -> String {
    let mut line = quoted(program);
    for arg in args {
        line.push(' ');
        line.push_str(&quoted(arg));
    }
    line
}

/// One argument, safe to paste into a `bash` script.
///
/// **Deliberately not a shell-words library.** The set of characters that may go bare is small
/// and written down here, so everything else — a space, a `$`, a quote, a newline in a path —
/// takes the single-quote form, whose only escaping rule is `'\''`. A generous rule is what
/// breaks a script on the one machine whose home directory has a space in it.
fn quoted(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '@' | '%' | '_' | '+' | '=' | ':' | ',' | '.' | '/' | '-')
        });
    if plain {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', r"'\''"))
}

/// The artifact formats `shall export` can produce (II.94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bash,
}

impl Format {
    /// Every word `--format` takes for an artifact.
    ///
    /// The words `brew`, `pip`, `npm` and `apt` are the *other* export — a native manifest for
    /// one ecosystem, built from the state — and are parsed by [`crate::app::export::Format`].
    /// An export that writes a manager's own file and an export that writes a script *calling*
    /// that manager are two answers, so they are two parsers and one is never guessed from the
    /// other.
    pub const WORDS: &'static [&'static str] = &["bash"];

    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "bash" => Some(Self::Bash),
            _ => None,
        }
    }

    /// The word this format answers to.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "bash",
        }
    }
}

/// Where an artifact's truth comes from (#67, #66).
///
/// **Two sources, not two flags for one thing.** `Declarations` is what the machine is meant to
/// be and `Scan` is what it is; they disagree on most machines — twenty installed packages
/// nothing declares, and every `absent:` line invisible to a scan by construction — so a default
/// would answer one question while looking like the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Declarations,
    Scan,
}

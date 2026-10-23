use clap::{CommandFactory, Parser};
use std::collections::{HashMap, HashSet};

/// How much Shall says about itself, from argv alone.
pub fn log_level_from_argv(argv: &[String]) -> &'static str {
    let mut verbosity = 0u8;
    for arg in argv.iter().skip(1) {
        match arg.as_str() {
            "--quiet" => return "error",
            "--verbose" => verbosity += 1,
            "--" => break,
            _ if arg.starts_with('-') && !arg.starts_with("--") && arg.len() == 2 => {
                if arg.contains('q') {
                    return "error";
                }
                verbosity += arg.matches('v').count() as u8;
            }
            _ if arg.starts_with('-') && !arg.starts_with("--") => {
                if arg.starts_with("-c") {
                    continue;
                }
                if arg.contains('q') {
                    return "error";
                }
                verbosity += arg.matches('v').count() as u8;
            }
            _ => {}
        }
    }
    match verbosity {
        0 => "warn",
        1 => "info",
        _ => "debug",
    }
}

/// Global flags that take a separate-argument value, asked of clap rather than hand-listed.
pub fn global_value_flags() -> HashSet<String> {
    let mut out = HashSet::new();
    for a in <crate::cli::Cli as CommandFactory>::command().get_arguments() {
        if !matches!(
            a.get_action(),
            clap::ArgAction::Set | clap::ArgAction::Append
        ) {
            continue;
        }
        if let Some(l) = a.get_long() {
            out.insert(format!("--{}", l));
        }
        if let Some(c) = a.get_short() {
            out.insert(format!("-{}", c));
        }
    }
    out
}

/// Index of the subcommand token in argv, skipping leading global flags and their values.
pub fn find_subcommand_index(argv: &[String]) -> Option<usize> {
    let value_flags = global_value_flags();
    let mut i = 1;
    while i < argv.len() {
        let a = &argv[i];
        if a == "--" {
            return if i + 1 < argv.len() {
                Some(i + 1)
            } else {
                None
            };
        }
        if a.starts_with('-') {
            if value_flags.contains(a.as_str()) {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            return Some(i);
        }
    }
    None
}

/// Rewrite argv by expanding a user command alias in the subcommand slot.
pub fn expand_command_aliases(
    argv: Vec<String>,
    aliases: &HashMap<String, String>,
    known: &HashSet<String>,
) -> Vec<String> {
    let Some(idx) = find_subcommand_index(&argv) else {
        return argv;
    };
    let cmd = &argv[idx];
    if known.contains(cmd) {
        return argv;
    }
    if let Some(expansion) = aliases.get(cmd) {
        let mut out = Vec::with_capacity(argv.len() + 2);
        out.extend(argv[..idx].iter().cloned());
        out.extend(expansion.split_whitespace().map(|s| s.to_string()));
        out.extend(argv[idx + 1..].iter().cloned());
        return out;
    }
    argv
}

/// Plan a user-defined verb into the per-step argv it runs.
pub fn plan_user_verb(
    argv: &[String],
    verbs: &HashMap<String, Vec<String>>,
    known: &HashSet<String>,
) -> Option<Result<Vec<Vec<String>>, String>> {
    let idx = find_subcommand_index(argv)?;
    let cmd = &argv[idx];
    if known.contains(cmd) {
        return None;
    }
    let steps = verbs.get(cmd)?;
    if argv.len() > idx + 1 {
        return Some(Err(format!(
            "the verb `{}` takes no arguments, but `{}` was given.\n  \
             A verb is a fixed sequence of built-in commands (U35). To vary a step, edit the \
             `[verbs]` entry.",
            cmd,
            argv[idx + 1..].join(" ")
        )));
    }

    let leading = &argv[1..idx];
    let mut planned = Vec::with_capacity(steps.len());
    for step in steps {
        let tokens: Vec<String> = step.split_whitespace().map(|s| s.to_string()).collect();
        let Some(first) = tokens.first() else {
            return Some(Err(format!(
                "the verb `{}` has an empty step. Every step must be a built-in command.",
                cmd
            )));
        };
        if !known.contains(first) {
            return Some(Err(format!(
                "the verb `{}` step `{}` is not a built-in command.\n  \
                 A user verb may only compose built-in commands (U35). Running an arbitrary \
                 command from a verb is `exec:`'s job and is off by default (U33).",
                cmd, first
            )));
        }
        if let Some(flag) = first_global_flag(&tokens[1..], &global_posture_flags()) {
            return Some(Err(format!(
                "the verb `{}` step `{}` carries `{}`, which is a global flag.\n  \
                 A step's own flags configure that step (`upgrade --all`); the run's posture \
                 belongs before the verb name, where it applies to every step: \
                 `shall{} {}`. On a step it was silently dropped.",
                cmd,
                tokens.join(" "),
                flag,
                if leading.is_empty() {
                    String::new()
                } else {
                    format!(" {}", leading.join(" "))
                },
                cmd
            )));
        }
        let mut one = Vec::with_capacity(1 + leading.len() + tokens.len());
        one.push(argv[0].clone());
        one.extend(leading.iter().cloned());
        one.extend(tokens.clone());
        if let Err(e) = crate::cli::Cli::try_parse_from(&one) {
            return Some(Err(format!(
                "the verb `{}` step `{}` is not a valid command line: {e}",
                cmd,
                tokens.join(" ")
            )));
        }
        planned.push(one);
    }
    Some(Ok(planned))
}

/// The run-posture flag spellings, asked of clap rather than listed by hand.
pub fn global_posture_flags() -> HashSet<String> {
    let mut out = HashSet::new();
    for a in <crate::cli::Cli as CommandFactory>::command().get_arguments() {
        if !a.is_global_set() {
            continue;
        }
        if let Some(l) = a.get_long() {
            out.insert(format!("--{l}"));
        }
        if let Some(c) = a.get_short() {
            out.insert(format!("-{c}"));
        }
    }
    out
}

fn first_global_flag(tokens: &[String], globals: &HashSet<String>) -> Option<String> {
    for tok in tokens {
        let bare = if let Some(rest) = tok.strip_prefix("--") {
            Some(format!("--{}", rest.split('=').next().unwrap_or(rest)))
        } else if let Some(shorts) = tok.strip_prefix('-').filter(|s| !s.is_empty()) {
            shorts.chars().next().map(|c| format!("-{c}"))
        } else {
            None
        };
        if let Some(name) = bare {
            if globals.contains(&name) {
                return Some(tok.clone());
            }
        }
    }
    None
}

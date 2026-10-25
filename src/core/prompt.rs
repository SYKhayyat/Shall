//! Asking a human, in the one place that knows nobody may be there.
//!
//! Six call sites wrote the same three steps: honour `--yes`, check `stdin().is_terminal()`,
//! then `dialoguer::Confirm::new().default(false).interact()`. The steps are not the
//! interesting part — **the third answer is**. A confirm has three outcomes, not two: yes, no,
//! and *there is nobody to ask*, and every copy of these three steps got to decide the third
//! for itself. `snapshot_restore`'s gallery decided it by not deciding, and answered a restore
//! with `dialoguer`'s bare `IO error: not a terminal`.
//!
//! There are two right answers to *nobody is there*, and which one a site wants is the only
//! thing it has to say: refuse by name, or decline and carry on. Both are spelled out at the
//! call site because both are a policy about the machine, not a detail of the prompt.

use std::io::IsTerminal;

use crate::core::{Error, Result};

/// What a prompt does when there is no terminal on the other end.
pub enum Unattended<'a> {
    /// Refuse, with this sentence. For anything that changes the machine: a run that would
    /// have asked and cannot must stop, and must say what to pass instead of a human.
    ///
    /// The sentence is the site's own because a generic one helps nobody — it has to name the
    /// verb, and the flag, and the safe way to look first.
    Refuse(&'a str),
    /// Print this and answer *no*. For an offer the run is better off without but can survive:
    /// installing a missing package manager, running a prerequisite setup command.
    Decline(&'a str),
}

/// Ask `prompt`, unless `yes` was passed or nobody is there to answer.
///
/// The default is **no**. A confirm that defaults to yes is a confirm that fires on a stray
/// newline, and every one of these guards something the user did not spell out.
pub fn confirm(yes: bool, prompt: &str, unattended: Unattended<'_>) -> Result<bool> {
    confirm_in(std::io::stdin().is_terminal(), yes, prompt, unattended)
}

/// The seam the tests drive, so third-answer coverage does not depend on how the suite was
/// launched — a plain `cargo test` on a console *has* a terminal on stdin.
/// **`pub` for the integration tests' sake.** The mirror must drive the non-terminal seam without
/// depending on the test process's actual stdin.
#[doc(hidden)]
pub fn confirm_in(
    attended: bool,
    yes: bool,
    prompt: &str,
    unattended: Unattended<'_>,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    if !attended {
        return match unattended {
            Unattended::Refuse(why) => Err(Error::Refused(why.to_string())),
            Unattended::Decline(say) => {
                println!("{say}");
                Ok(false)
            }
        };
    }
    // The wait here is a person, and it is the longest wait Shall ever does — a confirm sits at
    // the prompt until someone answers, or until they walk away and never do. Reached from an
    // `async fn`, a bare `interact()` parks a tokio worker for the whole of it.
    Ok(crate::core::on_the_terminal(|| {
        dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()
    })?)
}

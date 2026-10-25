//! Who the answer is for — the reader, not a flag.
//!
//! `--json` reached the verbs as a bare `bool` threaded through twenty signatures, and three of
//! those signatures took more than one bool: `handle_sync(app, locked, upgrade, json)` is three
//! positional booleans in a row, where transposing any two compiles and changes what the command
//! does. `handle_install(app, packages, json, temp, into)` is the same shape one argument along.
//!
//! The second cost is that every site re-derived *human* by negating *machine*. `!json` appears
//! wherever a sentence is printed, which reads as "not machine-readable" rather than "there is a
//! person here", and it is why `sync --dry-run --json` on a converged machine printed the words
//! `already up to date` on the branch a healthy fleet takes: the early return was written under
//! `!json` reasoning and the document lived past it.
//!
//! One type, converted once, at the one place a flag exists.

/// Which reader this command is answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Output {
    /// A person is reading. Sentences, colour, progress, "nothing to do".
    #[default]
    Human,
    /// A machine is reading, over SSH in `fleet`'s case. One document on stdout and nothing
    /// else — a stray sentence makes the whole answer unparseable.
    Json,
}

impl Output {
    /// From the `--json` flag of whichever subcommand parsed it. This is the only conversion:
    /// a verb receives the decision, never the flag.
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            Self::Json
        } else {
            Self::Human
        }
    }

    pub fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }

    /// True when a sentence on stdout is wanted. The affirmative spelling of what used to be
    /// `!json`, so a reader does not have to negate their way to "a person is here".
    pub fn is_human(self) -> bool {
        matches!(self, Self::Human)
    }
}

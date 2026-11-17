//! `shall export --help` is the only place a user learns what `--format` takes, so it has to
//! name every word `parse` accepts — which it did not: the clap doc comment said
//! `One of: brew | pip | npm | apt (manifests) or bash (a script)`, four of the five manifests
//! and no alias, while `Format::parse` accepted `nix` (the NixOS module, `J5`) and six aliases
//! (#191). A doc comment is checked by nothing, so it drifted; the help has to come from the
//! same list the parser reads.
//!
//! The fix is one table: `Format::NAMES` holds every (format, words) pair, and `parse`,
//! `Format::WORDS`, `Format::HELP` and the refusal hint in `src/main.rs` all read it. These
//! tests hold the help against the table, in both directions: a word that parses but is not
//! advertised is #191 again, and a word that is advertised but does not parse would confuse a
//! user into a failure the help promised away.

use crate::harness::Fixture;
use shall::app::bootstrap;
use shall::app::export::Format;

/// The table is the vocabulary: every word parses to the format it names, the canonical word
/// round-trips, and `WORDS` is exactly the canonicals in order.
#[test]
fn the_table_is_the_vocabulary() {
    for (format, words) in Format::NAMES {
        for word in words {
            assert_eq!(
                Format::parse(word),
                Some(format),
                "`{word}` must parse to {format:?} — the table and the parser disagree"
            );
        }
        assert_eq!(
            Format::parse(format.as_str()),
            Some(format),
            "the canonical word `{}` must parse back to its format",
            format.as_str()
        );
    }
    let canonical: Vec<&str> = Format::all().iter().map(|f| f.as_str()).collect();
    assert_eq!(
        canonical,
        Format::WORDS.to_vec(),
        "WORDS must be the canonicals, in order"
    );
    assert_eq!(
        Format::parse("haskell"),
        None,
        "nothing outside the table parses"
    );
}

/// The help text names every manifest word and every alias, plus the script formats — a word
/// that parses but is unadvertised is exactly how #191 happened.
#[test]
fn the_help_names_every_format_and_every_alias() {
    let help = Format::HELP.to_ascii_lowercase();
    for (format, words) in Format::NAMES {
        for word in words {
            assert!(
                help.contains(word),
                "`{word}` ({format:?}) parses but is not named in the --help text: {help}"
            );
        }
    }
    // A script format lives in the other parser (`bootstrap::Format`) but shares the one
    // `--format` flag, so it has to be advertised in the one help line.
    for word in bootstrap::Format::WORDS {
        assert!(
            help.contains(word),
            "`{word}` (a script format) is not named in the --help text: {help}"
        );
    }
}

/// The help a user actually sees is the const, not a better const that got disconnected.
#[test]
fn the_emitted_help_names_every_word() {
    let (out, code) = Fixture::new("export-help").run(&["export", "--help"]);
    let lower = out.to_ascii_lowercase();
    for (_, words) in Format::NAMES {
        for word in words {
            assert!(
                lower.contains(word),
                "`export --help` (code {code}) does not name `{word}`:\n{out}"
            );
        }
    }
}

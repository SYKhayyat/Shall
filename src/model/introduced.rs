//! When a declaration first appeared, and in which commit (XIII.19).
//!
//! **Git already knows.** The config repo is a git repo (II.1), every sync commits, and the
//! commit that first added a line is a fact already recorded — so the answer is a question put
//! to git, not a second history Shall writes at sync time and has to keep true. A store
//! recording "when did this line appear" would be a copy of git's answer that can disagree with
//! it, and the copy is always the one that is wrong.
//!
//! Pure: the argv, and parsing what git says. Running it is the caller's.

/// The commit that introduced a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Introduced {
    pub commit: String,
    /// Author date, as git formatted it (`%ad` with `--date=short`).
    pub date: String,
    pub subject: String,
}

impl Introduced {
    /// One line for a human: `a1b2c3d 2026-03-14  add ripgrep for the new grep workflow`.
    pub fn summary(&self) -> String {
        format!("{}  {}  {}", self.short(), self.date, self.subject)
    }

    /// The abbreviated hash people actually type.
    pub fn short(&self) -> &str {
        let n = self.commit.len().min(7);
        &self.commit[..n]
    }
}

/// The separator between fields. A tab cannot appear in a commit hash or a `--date=short` date,
/// and a subject containing one still parses because the split is bounded to two.
const SEP: char = '\t';

/// Ask git which commits touched a line naming `needle`, oldest last.
///
/// `-S` is git's pickaxe: it finds commits where the *number of occurrences* of the string
/// changed, which is exactly "the commit that added this line" and not "every commit that
/// touched this file". `--` limits it to the config repo's declaration files, so a mention in
/// a README is not mistaken for a declaration.
pub fn argv(needle: &str, paths: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = [
        "log",
        "--format=%H\t%ad\t%s",
        "--date=short",
        "-S",
        needle,
        "--",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    out.extend(paths.iter().map(|p| p.to_string()));
    out
}

/// The commit that INTRODUCED the declaration, from `git log`'s output.
///
/// git lists newest first, so the introducing commit is the **last** line. Taking the first
/// would name the most recent commit that touched the line — which is the commit that last
/// edited it, a different and much less interesting fact, and one that would silently be right
/// whenever a line has only ever been touched once.
pub fn introduced_in(git_log: &str) -> Option<Introduced> {
    git_log
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .and_then(parse_line)
}

fn parse_line(line: &str) -> Option<Introduced> {
    let mut parts = line.splitn(3, SEP);
    let commit = parts.next()?.trim();
    let date = parts.next()?.trim();
    // A commit with an empty subject is legal, so this is the one field allowed to be missing.
    let subject = parts.next().unwrap_or("").trim();
    if commit.is_empty() || date.is_empty() {
        return None;
    }
    Some(Introduced {
        commit: commit.to_string(),
        date: date.to_string(),
        subject: subject.to_string(),
    })
}

//! Choosing which file of a release to install.
//!
//! Nothing here touches the network. The selector is given the assets a release offers and
//! returns the choice plus what it passed over, so the plan can show the decision and the lock
//! can record it. A selection that cannot be explained is a selection that drifts.

use super::format::{Format, FormatOrder};
use super::pattern::AssetPattern;
use super::platform::Platform;
use std::fmt;

/// One file a release offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

impl Asset {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Asset {
            name: name.into(),
            url: url.into(),
        }
    }
}

/// An asset the selector chose, with the format it was chosen as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub asset: Asset,
    pub format: Format,
}

/// An asset that could have been installed and was not. Only assets that survive the platform
/// and format filters appear here — the ones this machine cannot run are noise, not a choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassedOver {
    pub name: String,
    pub format: Format,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub picks: Vec<Pick>,
    pub passed_over: Vec<PassedOver>,
}

impl Selection {
    /// Whether the choice came down to a tie-break rather than being forced. The plan says so
    /// out loud when it did: a guess that is printed is not the silent guess.
    pub fn was_ambiguous(&self) -> bool {
        !self.passed_over.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rejection {
    ForeignPlatform,
    UnwantedFormat,
    UnmatchedPattern,
    Unrecognised,
}

impl Rejection {
    fn describe(self) -> &'static str {
        match self {
            Rejection::ForeignPlatform => "not for this machine",
            Rejection::UnwantedFormat => "format not in your list",
            Rejection::UnmatchedPattern => "does not match @asset",
            Rejection::Unrecognised => "unrecognised file type",
        }
    }
}

#[derive(Debug, Clone)]
struct Rejected {
    name: String,
    format: Option<Format>,
    why: Rejection,
}

/// Everything the selector needs, and nothing it does not. Constructed by the caller so this
/// module never reaches for global state or the environment.
pub struct Request<'a> {
    pub package: &'a str,
    pub release: &'a str,
    pub platform: &'a Platform,
    pub formats: &'a FormatOrder,
    pub pattern: Option<&'a AssetPattern>,
}

#[derive(Debug, Clone)]
pub struct NoMatch {
    package: String,
    release: String,
    formats: String,
    pattern: Option<String>,
    rejected: Vec<(String, Option<Format>, &'static str)>,
}

impl fmt::Display for NoMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} — no asset matches your formats.", self.package)?;
        writeln!(f, "  wanted:  {}", self.formats)?;
        if let Some(p) = &self.pattern {
            writeln!(f, "  @asset:  {}", p)?;
        }
        if self.rejected.is_empty() {
            writeln!(f, "  release {} offers no assets at all.", self.release)?;
        } else {
            writeln!(f, "  release {} offers:", self.release)?;
            let width = self
                .rejected
                .iter()
                .map(|(n, _, _)| n.len())
                .max()
                .unwrap_or(0);
            for (name, format, why) in &self.rejected {
                let shape = format.map(|f| f.to_string()).unwrap_or_else(|| "-".into());
                writeln!(
                    f,
                    "    {:width$}  {:<9} ({})",
                    name,
                    shape,
                    why,
                    width = width
                )?;
            }
        }
        write!(
            f,
            "  add the format you want to `priority`, or pin one with @formats= or @asset=."
        )
    }
}

impl std::error::Error for NoMatch {}

/// A candidate that survived every filter, carried with the facts the tie-break needs.
struct Candidate {
    asset: Asset,
    format: Format,
    rank: usize,
    specificity: u8,
}

pub fn select(request: &Request<'_>, assets: &[Asset]) -> Result<Selection, NoMatch> {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut rejected: Vec<Rejected> = Vec::new();

    for asset in assets {
        if Format::is_metadata_filename(&asset.name) {
            continue;
        }
        let runs_here = request.platform.accepts(&asset.name);
        let format = classify_format(&asset.name, request.platform, request.pattern);

        if !runs_here {
            rejected.push(Rejected {
                name: asset.name.clone(),
                format,
                why: Rejection::ForeignPlatform,
            });
            continue;
        }
        let Some(format) = format else {
            rejected.push(Rejected {
                name: asset.name.clone(),
                format: None,
                why: Rejection::Unrecognised,
            });
            continue;
        };
        if let Some(pattern) = request.pattern {
            if !pattern.matches(&asset.name) {
                rejected.push(Rejected {
                    name: asset.name.clone(),
                    format: Some(format),
                    why: Rejection::UnmatchedPattern,
                });
                continue;
            }
        }
        let Some(rank) = request.formats.rank(format) else {
            rejected.push(Rejected {
                name: asset.name.clone(),
                format: Some(format),
                why: Rejection::UnwantedFormat,
            });
            continue;
        };
        candidates.push(Candidate {
            asset: asset.clone(),
            format,
            rank,
            specificity: request.platform.specificity(&asset.name),
        });
    }

    if candidates.is_empty() {
        return Err(NoMatch {
            package: request.package.to_string(),
            release: request.release.to_string(),
            formats: request.formats.to_string(),
            pattern: request.pattern.map(|p| p.to_string()),
            rejected: rejected
                .into_iter()
                .map(|r| (r.name, r.format, r.why.describe()))
                .collect(),
        });
    }

    sort_by_preference(&mut candidates, request.formats.is_user_specified());

    if request
        .pattern
        .is_some_and(AssetPattern::installs_every_match)
    {
        return Ok(Selection {
            picks: candidates
                .into_iter()
                .map(|c| Pick {
                    asset: c.asset,
                    format: c.format,
                })
                .collect(),
            passed_over: Vec::new(),
        });
    }

    let mut candidates = candidates.into_iter();
    let winner = candidates.next().expect("candidates is non-empty");
    let passed_over = candidates
        .map(|c| PassedOver {
            name: c.asset.name,
            format: c.format,
        })
        .collect();

    Ok(Selection {
        picks: vec![Pick {
            asset: winner.asset,
            format: winner.format,
        }],
        passed_over,
    })
}

/// The tie-break, in one place so there is exactly one answer to "two assets, both legal".
///
/// Format order and specificity trade places depending on where the order came from (D2):
///
/// - **A `@formats=` the user wrote wins outright.** They named a preference; the asset highest
///   in it is the answer, and specificity only breaks a tie *within* one format.
/// - **A detected order yields to the machine.** The default order is Shall's guess about file
///   *type*, and an asset naming your exact os and arch is a stronger guess than one naming
///   neither — so specificity leads, and format order breaks ties within one specificity. This
///   is what stops jq's source `jq-1.8.2.tar.gz` beating `jq-linux-amd64` on a Linux box.
///
/// After that: the shortest filename, then alphabetical, so the result never depends on the
/// order GitHub happened to return.
fn sort_by_preference(candidates: &mut [Candidate], user_specified: bool) {
    candidates.sort_by(|a, b| {
        let rank = a.rank.cmp(&b.rank);
        let spec = b.specificity.cmp(&a.specificity);
        let primary = if user_specified {
            rank.then(spec)
        } else {
            spec.then(rank)
        };
        primary
            .then(a.asset.name.len().cmp(&b.asset.name.len()))
            .then(a.asset.name.cmp(&b.asset.name))
    });
}

/// An asset with no recognised extension is a `Binary` only if the filename **names this
/// machine** and its trailing dot-segment is not a file extension.
///
/// D2's rule is *"matched this machine's os/arch and has no recognised extension"*, and
/// "matched" is stronger than `accepts`, which only says the name does not contradict this
/// machine. Checked against real releases, the weaker reading made `MD5SUMS` — an actual asset
/// of every rclone release — an executable candidate on every platform, and `jq-linux64` one
/// on Windows. This is the one place in artifact selection that fails quietly rather than
/// loudly, so it takes the strict reading and a file that claims nothing is not an executable.
///
/// **`@asset=` overrides it**, because naming the file *is* the claim: a project shipping one
/// bare `mytool` for one platform is installed by naming it, not by Shall guessing.
///
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so the wider `pub(super)` left them with
/// nothing to call.
pub fn classify_format(
    name: &str,
    platform: &Platform,
    pattern: Option<&AssetPattern>,
) -> Option<Format> {
    if let Some(format) = Format::of_filename(name) {
        return Some(format);
    }
    if has_extension(name) {
        return None;
    }
    if platform.specificity(name) > 0 {
        return Some(Format::Binary);
    }
    if pattern.is_some_and(|p| p.names_exactly(name)) {
        return Some(Format::Binary);
    }
    None
}

fn has_extension(name: &str) -> bool {
    let tail = name.rsplit('/').next().unwrap_or(name);
    match tail.rsplit_once('.') {
        None => false,
        Some((stem, ext)) => {
            !stem.is_empty()
                && (1..=5).contains(&ext.len())
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
                && ext.chars().any(|c| c.is_ascii_alphabetic())
        }
    }
}

/// Which rule chose the artifact, in one phrase for `why` and the lock (D14).
///
/// The three places the answer can live, in the order they win: an explicit `@asset=` pattern
/// on the line, then a `@formats=` the user wrote, then the built-in default order detected from
/// the host. Naming which one won is the whole of D14 — when `github:x/y` installs a `.tar.gz`
/// on a machine the user expected a `.deb`, this says whether their line or Shall's default
/// decided it.
pub fn selection_reason(has_asset_pattern: bool, formats_user_specified: bool) -> &'static str {
    if has_asset_pattern {
        "your @asset= pattern"
    } else if formats_user_specified {
        "your @formats= line"
    } else {
        "the built-in default order for this machine"
    }
}

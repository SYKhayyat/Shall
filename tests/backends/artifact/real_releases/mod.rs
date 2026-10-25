//! D2: the classifier, run against the asset lists of real releases and checked by hand.
//!
//! The rule — an extension decides the format, and a name with no recognised extension that
//! matches this machine is `binary` — was ruled on reasoning and never tested against what
//! projects actually publish. It is the one part of artifact selection that fails quietly, so
//! the check is a fixture of real filenames rather than filenames someone imagined.

use shall::backends::artifact::format::{Format, FormatOrder};
pub use shall::backends::artifact::pattern::AssetPattern;
pub use shall::backends::artifact::platform::Platform;
pub use shall::backends::artifact::select::{classify_format, select, Asset, Request};

struct Block {
    repo: String,
    expected: Vec<(String, String, String)>,
    assets: Vec<Asset>,
}

fn blocks() -> Vec<Block> {
    let text = include_str!("real_releases.txt");
    let mut out: Vec<Block> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || (line.starts_with('#') && !line.starts_with("## ")) {
            continue;
        }
        if let Some(head) = line.strip_prefix("## ") {
            out.push(Block {
                repo: head.to_string(),
                expected: Vec::new(),
                assets: Vec::new(),
            });
        } else if let Some(exp) = line.strip_prefix("-> ") {
            let (target, want) = exp.split_once('=').expect("`-> os/arch = expected`");
            let (os, arch) = target.trim().split_once('/').expect("os/arch");
            let b = out.last_mut().expect("an expectation before any block");
            b.expected
                .push((os.to_string(), arch.to_string(), want.trim().to_string()));
        } else {
            let b = out.last_mut().expect("an asset before any block");
            b.assets
                .push(Asset::new(line, format!("https://example.test/{}", line)));
        }
    }
    assert!(out.len() >= 6, "the fixture lost its blocks");
    out
}

/// `linux` is checked as a debian machine because that is the default order that puts a system
/// package first, which is the one most likely to pick something surprising.
fn family_of(os: &str) -> Option<&'static str> {
    match os {
        "linux" => Some("debian"),
        _ => None,
    }
}

mod a_release_of_nothing_but_signatures_is_an_error;
mod every_real_release_resolves_to_the_asset_a_human_checked;
mod naming_an_asset_exactly_installs_it_even_when_the_name_says_nothing;
mod nothing_becomes_an_executable_without_naming_the_machine;
mod the_two_real_files_that_used_to_be_executables_are_not;

//! Reading the artifact options off a resolved `PackageSpec`.
//!
//! One place turns option strings into the selector's types, so a backend never parses
//! `formats` itself and the grammar's validation and the backend's reading cannot disagree.

use super::format::FormatOrder;
use super::pattern::AssetPattern;
use crate::config::grammar::Options;

#[derive(Debug, Default)]
pub struct ArtifactOptions {
    /// `None` means the user did not say, so the detected default applies.
    pub formats: Option<FormatOrder>,
    pub asset: Option<AssetPattern>,
    pub bin: Option<String>,
    /// `@download_only` (D3b): fetch the artifact and stop — never unpack it, shim it, or put it
    /// on `PATH`. It is still declared, so it is still removed when the line goes. This is what a
    /// download backend does *by default* when it has no way to install the thing it fetched (no
    /// executable inside, no installer), rather than failing.
    pub download_only: bool,
}

impl ArtifactOptions {
    /// **The list arrives as a list.** `formats` used to be one string that this function split
    /// on a `;` `to_spec` had joined it with — a delimiter nothing validated, standing in for a
    /// `Vec` the grammar had produced and then thrown away. A `@formats=` value containing a
    /// semicolon was unrepresentable and no layer said so.
    pub fn read(options: &Options) -> Result<Self, String> {
        let raw = options.all("formats");
        let formats = if raw.is_empty() {
            None
        } else {
            Some(
                FormatOrder::parse_all(
                    raw.iter()
                        .map(String::as_str)
                        .filter(|s| !s.trim().is_empty()),
                )
                .map_err(|e| e.to_string())?
                // A `@formats=` the user wrote is an instruction, and the tie-break honours
                // it over an asset that merely names the machine well (D2).
                .as_user_specified(),
            )
        };

        let asset = match options.one("asset") {
            None => None,
            Some(raw) => Some(AssetPattern::parse(raw).map_err(|e| e.to_string())?),
        };

        let bin = options
            .one("bin")
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty());

        let download_only = options
            .one("download_only")
            .map(|v| v != "false" && v != "no")
            .unwrap_or(false);

        Ok(ArtifactOptions {
            formats,
            asset,
            bin,
            download_only,
        })
    }

    /// The line beats `priority`, and `priority` beats the built-in default. A line's list
    /// replaces rather than extends: half-overriding an ordered list produces an order nobody
    /// wrote.
    pub fn resolved_formats(&self, fallback: &FormatOrder) -> FormatOrder {
        match &self.formats {
            Some(f) if !f.is_empty() => f.clone(),
            _ => fallback.clone(),
        }
    }
}

/// The built-in default for this machine, from the same detected facts `when family` reads.
pub fn default_formats() -> FormatOrder {
    let facts = crate::config::parser::HostFacts::current();
    FormatOrder::detected_default(&facts.os, Some(facts.family.as_str()))
}

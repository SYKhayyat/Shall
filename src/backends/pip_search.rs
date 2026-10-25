//! Searching PyPI, which is a `SearchSource` rather than a backend of its own.
//!
//! The legacy `pip search` was disabled upstream — PyPI withdrew the XML-RPC endpoint over
//! abuse — and there is no public full-text replacement. So `search` is an EXACT-NAME lookup
//! against `https://pypi.org/pypi/<name>/json`: the package if it exists, nothing if it does
//! not. A documented limitation: name resolution, not discovery.
//!
//! **This used to be a bespoke `impl Searchable` bolted onto pip's registration**, which meant
//! *a search that is an HTTP call* was a thing only Rust could say — `node_registry.rs` had the
//! same shape and got a `SearchSource` variant, and pip did not. A row can now say
//! `search_source = "pypi"` for the same reason it can say `"npm_registry"`.

use crate::backends::node_registry::http_timeout;
use crate::core::{Error, Package, Result};

/// Resolve `query` against PyPI, tagging the result with `backend`.
pub async fn registry_search(query: &str, backend: &str) -> Result<Vec<Package>> {
    let name = query.trim();
    if name.is_empty() {
        return Ok(vec![]);
    }

    let client = crate::core::http::api("shall-manager", http_timeout().as_secs())?;

    // Path-segment encoding, hand-rolled rather than a dependency for one call site: a
    // package name is `[A-Za-z0-9._-]`, everything else (a stray `/`, space or `?` from user
    // text) is path syntax to the server and gets escaped.
    let encoded = percent_encode_segment(name);
    let url = format!("https://pypi.org/pypi/{encoded}/json");
    let res = client.get(&url).send().await.map_err(Error::from)?;

    // 404 simply means "no such package" — not an error for a search.
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(vec![]);
    }
    if !res.status().is_success() {
        return Err(Error::Other(format!("PyPI API error: {}", res.status())));
    }

    let json: serde_json::Value = res.json().await.map_err(Error::from)?;
    Ok(vec![parse_pypi(&json, name, backend)])
}

/// Percent-encode one URL path segment: unreserved bytes pass, everything else becomes `%XX`.
pub fn percent_encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Parse a PyPI JSON document (`https://pypi.org/pypi/<name>/json`) into a `Package`.
/// `fallback_name` is used if the document omits `info.name`.
pub fn parse_pypi(json: &serde_json::Value, fallback_name: &str, backend: &str) -> Package {
    let info = &json["info"];
    let pkg_name = info["name"].as_str().unwrap_or(fallback_name);
    let mut p = Package::new(pkg_name, backend);
    if let Some(v) = info["version"].as_str() {
        p.version = Some(v.to_string());
    }
    if let Some(d) = info["summary"].as_str() {
        if !d.is_empty() {
            p.properties
                .insert("description".to_string(), d.to_string());
        }
    }
    p
}

//! The embedded `vars.shall` provider (Part IX): a script Shall runs in-process to produce
//! `name → value` pairs, in a language it ships so a fleet resolves identically with nothing to
//! install. Rhai is the engine, behind the neutral `vars.shall` extension so it can be replaced
//! without renaming anyone's files.
//!
//! **The engine is not sandboxed, and that is the ruling.** A stock Rhai `Engine` has no file,
//! shell, clock or network access; `core::rhai_stdlib` puts all four back, always on, because
//! II.6b decided `vars.shall` is *"trusted the same as a hook — a script in your own repo"* and
//! gave it every power an external `vars.py` already had. A `#rhai` hook builds its engine from
//! that same function, so the two are one language and cannot drift apart.
//!
//! **What makes that safe is the ledger, not the engine.** The file is hashed into `locks/` and
//! goes through II.12: first sight asks, a changed hash stops, and under `-y` or with no
//! terminal an unapproved provider is a refusal rather than a skipped prompt. That matters more
//! here than for a hook, because this file resolves at **step 0** of II.7 — before a plan
//! exists — so `check`, `plan` and even `plan --dry-run` have already run it by the time they
//! print anything. *"I only previewed it"* is not a state in which this script has not run.

use crate::config::grammar::{GrammarError, Origin, Result};
use crate::config::parser::HostFacts;
use crate::model::vars::{Value, VarOrigins, Vars};
use rhai::{Dynamic, Scope};
use std::path::Path;

/// Run `vars.shall` and turn the map it evaluates to into resolved variables.
///
/// The script is handed the machine's detected facts as the constants `OS`, `ARCH`, `HOST`,
/// `FAMILY`, `HOME` and `USER`, and must end in a map: `#{ role: "travel", cores: 8 }`. The
/// map's values are the four types (string, number, boolean, list); a map value, or a script
/// that does not end in a map, is an error naming the file. A fact the machine could not
/// detect is absent rather than empty, so a script that reads it fails loudly instead of
/// resolving a variable to the wrong value with no sign it failed.
pub fn resolve(path: &Path, facts: &HostFacts) -> Result<Vars> {
    resolve_with_origins(path, facts).map(|(v, _)| v)
}

/// [`resolve`], plus where each variable came from. A script has no lines to attribute, so every
/// name points at the script file itself (W11/W12).
pub fn resolve_with_origins(path: &Path, facts: &HostFacts) -> Result<(Vars, VarOrigins)> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("vars.shall")
        .to_string();
    let origin = Origin::new(name.clone(), 0);

    let code = std::fs::read_to_string(path).map_err(|e| {
        GrammarError::new(origin.clone(), format!("could not read `{}`: {}", name, e))
    })?;

    let engine = crate::core::rhai_stdlib::engine("vars");

    let mut scope = Scope::new();
    scope.push_constant("OS", facts.os.clone());
    scope.push_constant("ARCH", facts.arch.clone());
    scope.push_constant("HOST", facts.host.clone());
    scope.push_constant("FAMILY", facts.family.clone());
    if let Some(home) = &facts.home {
        scope.push_constant("HOME", home.clone());
    }
    if let Some(user) = &facts.user {
        scope.push_constant("USER", user.clone());
    }

    let map = engine
        .eval_with_scope::<rhai::Map>(&mut scope, &code)
        .map_err(|e| {
            GrammarError::new(origin.clone(), format!("`{}` did not run: {}", name, e)).with_hint(
                "the script must end in a map of name → value, e.g. `#{ role: \"travel\" }`.",
            )
        })?;

    let mut vars = Vars::new();
    let mut origins = VarOrigins::new();
    for (key, value) in map {
        let key = key.to_string();
        valid_name(&key, &origin)?;
        origins.insert(key.clone(), origin.clone());
        vars.insert(key, dynamic_to_value(value, &origin)?);
    }
    Ok((vars, origins))
}

/// Rhai's types map onto ours; a map or a `()` is refused, because a variable is a scalar or a
/// list and a comparison would not know what to do with either.
fn dynamic_to_value(d: Dynamic, origin: &Origin) -> Result<Value> {
    if d.is_bool() {
        return Ok(Value::Bool(d.as_bool().unwrap_or(false)));
    }
    if d.is_int() {
        return Ok(Value::Num(d.as_int().unwrap_or(0) as f64));
    }
    if d.is_float() {
        return Ok(Value::Num(d.as_float().unwrap_or(0.0)));
    }
    if d.is_string() {
        return Ok(Value::Str(d.into_string().unwrap_or_default()));
    }
    if d.is_array() {
        let items = d.into_array().unwrap_or_default();
        return Ok(Value::List(
            items
                .into_iter()
                .map(|e| dynamic_to_value(e, origin))
                .collect::<Result<Vec<_>>>()?,
        ));
    }
    Err(GrammarError::new(
        origin.clone(),
        format!(
            "a variable cannot be a {}; it is a string, number, boolean, or list",
            d.type_name()
        ),
    ))
}

/// A variable name is an identifier, so it can be read back as `$name`.
fn valid_name(name: &str, origin: &Origin) -> Result<()> {
    let ok = !name.is_empty()
        && name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    if ok {
        Ok(())
    } else {
        Err(GrammarError::new(
            origin.clone(),
            format!("`{}` is not a valid variable name", name),
        ))
    }
}

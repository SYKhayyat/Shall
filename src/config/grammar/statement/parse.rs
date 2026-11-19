//! Reading one line into a [`Statement`], and the only `backend:name` parser there is.

use super::*;

/// Parse one statement. `line` must already have had comments stripped and be non-blank.
///
/// This is the only `backend:name` parser. Eight existed before, six of which never
/// checked that the prefix named a real backend — so every new prefix (`absent:`, `re:`,
/// `repo:`) was a thing they silently read as a backend name (C13).
pub fn parse(origin: &Origin, line: &str, backends: &dyn BackendNames) -> Result<Statement> {
    let stmt = parse_inner(origin, line, backends)?;
    validate(origin, &stmt)?;
    Ok(stmt)
}

fn parse_inner(origin: &Origin, line: &str, backends: &dyn BackendNames) -> Result<Statement> {
    let line = line.trim();

    if let Some(rest) = line.strip_prefix("use ") {
        return parse_use(origin, rest.trim());
    }
    if line == "use" || line.starts_with("use\t") {
        return parse_use(origin, line[3..].trim());
    }

    // `param NAME` / `param NAME = DEFAULT` (U32). Checked before the package parser so a bare
    // `param gpu` is a parameter declaration, not a package named `param gpu`.
    if let Some(rest) = line.strip_prefix("param ") {
        return parse_param(origin, rest.trim());
    }
    if line == "param" || line.starts_with("param\t") {
        return parse_param(origin, line[5..].trim());
    }

    // II.4's set directives. Checked before the package parser, which would otherwise read
    // `exclude heavy` as a package named `exclude heavy`.
    for word in ["exclude ", "intersect "] {
        if let Some(rest) = line.strip_prefix(word) {
            return parse_set_directive(origin, word.trim(), rest.trim());
        }
    }

    // V.46: `use` already means union, so a second word for it is two ways to do one thing.
    if let Some(rest) = line.strip_prefix("include ") {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`include {}` — there is no `include`", rest.trim()),
        )
        .with_hint(format!(
            "write `use {}`. One word brings something in, everywhere: modules use it too.",
            rest.trim()
        )));
    }

    // A set expression, before the package parser reads `(Work` as a package name — but NOT
    // before the typed statements. `looks_like_expression` fires on `\ | & (`, and a
    // `link:C:\Users\me\.vimrc` is full of `\`: without this guard II.4's set math eats
    // II.2's statements, and `link:` silently parses as `Statement::Expr`. A line that opens
    // with a known statement prefix — or with a known backend, which makes it a package line —
    // is that statement, never an expression.
    if !starts_with_statement_prefix(line)
        && !opens_a_package_line(line, backends)
        && crate::app::profile_expr::looks_like_expression(line)
    {
        return Ok(Statement::Expr(line.to_string()));
    }

    // `-vim`. Checked after expressions so `a \ b` is a difference, not a subtraction.
    if let Some(rest) = line.strip_prefix('-') {
        let target = rest.trim();
        if target.is_empty() {
            return Err(GrammarError::new(origin.clone(), "`-` subtracts nothing")
                .with_hint("write `-vim` to take one package out."));
        }
        reject_leading_dash(origin, target)?;
        return Ok(Statement::Subtract(target.to_string()));
    }

    if let Some(rest) = line.strip_prefix("absent:") {
        let decl = parse_package(origin, rest.trim(), backends)?;
        if decl.backend.is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`absent:{}` does not name a backend",
                    decl.selector.as_str()
                ),
            )
            .with_hint(
                "an `absent:` line reaches outside what Shall manages, so it must say which \
                 backend: `absent:apt:libreoffice`.",
            ));
        }
        return Ok(Statement::Absent(decl));
    }

    if let Some(rest) = line.strip_prefix("repo:") {
        let rest = rest.trim();
        // `repo:apt:ppa:deadsnakes/ppa` — backend, then the spec (which has its own colons).
        let Some((backend, spec)) = rest.split_once(':') else {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`repo:{}` does not name a backend", rest),
            )
            .with_hint(
                "a repository belongs to one package manager, so name it: \
                 `repo:apt:ppa:deadsnakes/ppa`. A PPA is apt's, a COPR is dnf's.",
            ));
        };
        let (backend, spec) = (backend.trim(), spec.trim());
        if backend.is_empty() || spec.is_empty() {
            return Err(
                GrammarError::new(origin.clone(), "`repo:` needs `backend:spec`")
                    .with_hint("for example `repo:apt:ppa:deadsnakes/ppa`."),
            );
        }
        if !backends.is_backend(backend) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a backend", backend),
            )
            .with_hint("name the package manager that owns this repository, e.g. `apt`."));
        }
        reject_leading_dash(origin, spec)?;
        return Ok(Statement::Repo {
            backend: backend.to_string(),
            spec: spec.to_string(),
        });
    }

    for kw in KEYWORDS {
        let Some(build) = kw.build else { continue };
        if let Some(rest) = line.strip_prefix(kw.spelling) {
            let (name, options) = split_options(origin, rest.trim())?;
            if name.is_empty() {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`{}` names nothing", kw.spelling),
                ));
            }
            reject_leading_dash(origin, &name)?;
            return Ok(build(name, options));
        }
    }

    if let Some(var) = parse_var(line) {
        return Ok(var);
    }

    // Q16: a bare keyword is not a package name. Checked here, after every form that has a
    // delimiter has been dispatched, so it fires only on the line that would otherwise become
    // a package declaration.
    if let Some(kw) = bare_keyword(line) {
        return Err(keyword_is_not_a_package(origin, kw));
    }

    let decl = parse_package(origin, line, backends)?;
    Ok(Statement::Package(decl))
}

/// The keyword a line is nothing but, or `None`.
///
/// A line carrying a `:` is spelled with a prefix — `link:` is the statement and `list:link`
/// / `cargo:link` are the package — so it is never the bare form and is not this function's
/// business. Options are: `link @target=/x` and `link@version=1` are the same typo as `link`
/// with the rest of the line still attached, and mean a package no more than the bare word does.
fn bare_keyword(line: &str) -> Option<&'static Keyword> {
    if line.contains(':') {
        return None;
    }
    let head = line.split([' ', '\t', '@']).next()?;
    KEYWORDS.iter().find(|k| k.word() == head)
}

/// Q16, ruled 2026-07-30. Names both ways to mean the word, because both are reachable and
/// which one was meant is not knowable from the line.
fn keyword_is_not_a_package(origin: &Origin, kw: &Keyword) -> GrammarError {
    let word = kw.word();
    let statement = if kw.takes_colon() {
        format!("to declare that `{}`:", word)
    } else {
        format!("to write the `{}` you meant:", word)
    };
    GrammarError::new(
        origin.clone(),
        format!("`{}` is a keyword, not a package name", word),
    )
    .with_hint(format!(
        "{:<38}{}\n  {:<38}list:{}   (or pin one: cargo:{})",
        statement, kw.means, "to install a package by that name:", word, word,
    ))
}

/// `NAME = VALUE` (IX.2), where NAME is an identifier.
///
/// Checked last, and only for a bare identifier before the `=`, so nothing that is already a
/// package line can be read as a variable: `apt:foo@version=1.2` has a `:` and an `@` in its
/// head, and `-vim` does not start with a name character.
fn parse_var(line: &str) -> Option<Statement> {
    let (head, value) = line.split_once('=')?;
    let name = head.trim();
    if name.is_empty() || !name.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        return None;
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some(Statement::Var {
        name: name.to_string(),
        // Verbatim to end of line, trimmed — the same rule as a block-form option value.
        value: value.trim().to_string(),
    })
}

/// Whether a line opens with one of II.2's typed-statement prefixes. Such a line is that
/// statement and must not be mistaken for a set expression (II.4), whatever punctuation its
/// payload carries — a `link:` target is a path, not a difference.
fn starts_with_statement_prefix(line: &str) -> bool {
    KEYWORDS
        .iter()
        .any(|k| k.takes_colon() && line.starts_with(k.spelling))
}

/// Whether a line opens with `<known backend>:`, which makes it a package line (II.2) and not
/// set math (II.4). `\` is a difference operator *and* a legal character in a package name —
/// `winget list` reports 185 such names on a stock Windows box (`ARP\Machine\X64\Firefox`) —
/// so without this the commonest line in the language is read as profile algebra.
///
/// A real difference between two qualified packages (`apt:jq \ apt:vim`) still parses as one:
/// an operator stands apart from its operands and a name never does.
fn opens_a_package_line(line: &str, backends: &dyn BackendNames) -> bool {
    let head = match line.split_whitespace().next() {
        Some(h) => h,
        None => return false,
    };
    let Some((backend, rest)) = head.split_once(':') else {
        return false;
    };
    if rest.is_empty() || !backends.is_backend(backend) {
        return false;
    }
    !has_spaced_operator_outside_quotes(line)
}

/// Whether one of [`SPACED_OPERATORS`] occurs OUTSIDE a quoted span.
///
/// An operator stands apart from its operands — and a name never stands apart from its
/// quotes: `winget:"App Name | Pro"` carries the bytes ` | `, but they belong to the name,
/// and reading them as set algebra made that line unwritable in any module.
fn has_spaced_operator_outside_quotes(line: &str) -> bool {
    let mut in_quotes = false;
    let mut masked = String::with_capacity(line.len());
    for c in line.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            // The quote itself marks a boundary either way.
            masked.push(' ');
        } else if in_quotes {
            masked.push(' ');
        } else {
            masked.push(c);
        }
    }
    SPACED_OPERATORS.iter().any(|op| masked.contains(op))
}

/// The set operators as they are written between operands. Glued forms (`(a|b)&c`) are still
/// expressions — they cannot open with a backend prefix.
const SPACED_OPERATORS: [&str; 3] = [" \\ ", " | ", " & "];

fn parse_use(origin: &Origin, target: &str) -> Result<Statement> {
    if target.is_empty() {
        return Err(GrammarError::new(origin.clone(), "`use` names nothing")
            .with_hint("write `use editors` (a module) or `use Work` (a profile)."));
    }

    // Split off an optional `(args)` before validating the name, so a `/` inside an argument
    // value (`use m(path=/etc/foo)`) is not mistaken for the path in a `use` target (U32).
    let (name, args) = match target.split_once('(') {
        Some((name, rest)) => {
            let inner = rest.strip_suffix(')').ok_or_else(|| {
                GrammarError::new(
                    origin.clone(),
                    format!("`use {}` opens `(` but never closes it", target),
                )
                .with_hint("write `use workstation(user=shaul, gpu=nvidia)`.")
            })?;
            (name.trim(), parse_use_args(origin, inner)?)
        }
        None => (target, Vec::new()),
    };

    // `use` takes a name, never a path and never a URL (II.2). A file from the internet is
    // a fetch step that puts a module on disk; then you `use` it by name like everything
    // else.
    if name.contains('/') || name.contains('\\') || name.contains("://") {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` looks like a path or a URL", name),
        )
        .with_hint(
            "`use` takes a name. Fetch the file into `modules/` first, then `use` it by name.",
        ));
    }
    if name.split_whitespace().count() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` names more than one thing", name),
        )
        .with_hint("one `use` per line."));
    }
    let reference = Reference::classify(name).ok_or_else(|| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is neither a module nor a profile name", name),
        )
        .with_hint("profiles are Capitalized, modules are lowercase.")
    })?;
    // A profile has no parameters to bind (U32): only modules declare `param`.
    if !args.is_empty() && matches!(reference, Reference::Profile(_)) {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` passes arguments to a profile", name),
        )
        .with_hint("only a module takes parameters (`param`); a profile has none to bind."));
    }
    Ok(Statement::Use(reference, args))
}

/// Parse `k=v, k2=v2` from inside a `use name(...)`. Values are verbatim to the next comma,
/// trimmed. An empty argument list (`use m()`) is allowed and binds nothing.
fn parse_use_args(origin: &Origin, inner: &str) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for piece in inner.split(',') {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((k, v)) = piece.split_once('=') else {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a `name=value` argument", piece),
            )
            .with_hint("write each argument as `name=value`, comma-separated."));
        };
        let key = k.trim();
        if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a parameter name", key),
            )
            .with_hint("parameter names are letters, digits and `_`."));
        }
        out.push((key.to_string(), v.trim().to_string()));
    }
    Ok(out)
}

/// `param NAME` / `param NAME = DEFAULT` (U32). The name is an identifier; a default is
/// verbatim to end of line, trimmed, exactly like a `Var` value.
fn parse_param(origin: &Origin, rest: &str) -> Result<Statement> {
    if rest.is_empty() {
        return Err(GrammarError::new(origin.clone(), "`param` names nothing")
            .with_hint("write `param user` (required) or `param gpu = none` (with a default)."));
    }
    let (name, default) = match rest.split_once('=') {
        Some((n, d)) => (n.trim(), Some(d.trim().to_string())),
        None => (rest.trim(), None),
    };
    if name.is_empty()
        || !name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{}` is not a parameter name", name),
        )
        .with_hint(
            "parameter names start with a letter or `_` and hold letters, digits and `_`.",
        ));
    }
    Ok(Statement::Param {
        name: name.to_string(),
        default,
    })
}

/// `exclude heavy` / `intersect security` — both take one module or profile name, and case
/// says which, exactly as `use` does.
fn parse_set_directive(origin: &Origin, word: &str, target: &str) -> Result<Statement> {
    if target.is_empty() {
        return Err(
            GrammarError::new(origin.clone(), format!("`{}` names nothing", word)).with_hint(
                format!(
                    "write `{} heavy` (a module) or `{} Work` (a profile).",
                    word, word
                ),
            ),
        );
    }
    if target.split_whitespace().count() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{} {}` names more than one thing", word, target),
        )
        .with_hint(format!("one `{}` per line.", word)));
    }
    let reference = Reference::classify(target).ok_or_else(|| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is neither a module nor a profile name", target),
        )
        .with_hint("profiles are Capitalized, modules are lowercase.")
    })?;
    Ok(match word {
        "exclude" => Statement::Exclude(reference),
        _ => Statement::Intersect(reference),
    })
}

/// Split `NAME@opts` into its name and options. Used by the non-package statements, whose
/// names are not `backend:name` (`shim:jq@source=cargo:jq`).
fn split_options(origin: &Origin, text: &str) -> Result<(String, Options)> {
    match text.split_once('@') {
        Some((name, opts)) => Ok((name.trim().to_string(), parse_short(origin, opts)?)),
        None => Ok((text.to_string(), Options::default())),
    }
}

/// Read the `backend:` prefix of a package line, which may name a chain.
///
/// `apt` pins. `apt,dnf` and `apt,list` do not — they say what the line will accept, in
/// order, and leave the choosing to the machine. A comma rather than a hyphen because
/// package managers have hyphens in their names (`nix-env`, `apt-get`), and a separator a
/// backend name can contain is a separator that stops working the day such a backend is
/// added.
fn parse_prefix(
    origin: &Origin,
    prefix: &str,
    backends: &dyn BackendNames,
) -> Result<(Option<String>, Candidates)> {
    // A group is a shortcut for a comma-chain (U18), so expand it into that chain BEFORE any
    // validation — every member then goes through the same backend check and `list`-only rules a
    // hand-written chain does. `tools:rg` becomes `apt,dnf,cargo:rg`, and `tools,brew:rg` splices
    // the group's members in front of `brew`. Nested groups are already flattened to terminal
    // backends by `Groups` (a cycle was refused at load), so a single expansion here is complete.
    let mut expanded: Vec<String> = Vec::new();
    for raw in prefix.split(',').map(str::trim) {
        match backends.expand_group(raw) {
            Some(members) => expanded.extend(members),
            None => expanded.push(raw.to_string()),
        }
    }
    let parts: Vec<&str> = expanded.iter().map(String::as_str).collect();

    let unknown = |name: &str| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is not a backend Shall uses", name),
        )
        .with_hint(format!(
            "add `{}` to your `priority` file, or check the spelling. Not listed means \
             Shall does not use it at all.",
            name
        ))
    };

    // `list` is only a tail: everything after it would never be reached, and writing
    // something unreachable means the line does not say what its author thinks it says.
    if let Some(pos) = parts.iter().position(|p| *p == PRIORITY_KEYWORD) {
        if pos != parts.len() - 1 {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` must come last in `{}`", PRIORITY_KEYWORD, prefix),
            )
            .with_hint(format!(
                "`{}` already means every manager in `priority`, so nothing written after \
                 it can ever be reached.",
                PRIORITY_KEYWORD
            )));
        }
    }

    let mut named: Vec<String> = Vec::new();
    let mut ends_in_priority = false;
    for part in &parts {
        if part.is_empty() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` has an empty backend in it", prefix),
            )
            .with_hint("write `apt,dnf:rg` — one manager between each comma."));
        }
        if *part == PRIORITY_KEYWORD {
            ends_in_priority = true;
            continue;
        }
        if !backends.is_backend(part) {
            return Err(unknown(part));
        }
        if named.iter().any(|n| n == part) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is named twice in `{}`", part, prefix),
            )
            .with_hint("the first one already decides; the second can never be reached."));
        }
        named.push(part.to_string());
    }

    Ok(match (named.len(), ends_in_priority) {
        // `list:rg`, which is what a bare `rg` means spelled out.
        (0, true) => (None, Candidates::Priority),
        (0, false) => return Err(unknown(prefix)),
        // One manager and no tail is the pin: apt or nothing.
        (1, false) => (Some(named.remove(0)), Candidates::Priority),
        (_, false) => (None, Candidates::Named(named)),
        (_, true) => (None, Candidates::NamedThenPriority(named)),
    })
}

/// A name reaches a manager's command line, where a leading `-` is an option and not a name.
/// The `--` every invocation emits (II.12b) holds for managers that honour it; this holds for
/// the rest, and it is the layer that can say *which line* is wrong.
fn reject_leading_dash(origin: &Origin, name: &str) -> Result<()> {
    if name.starts_with('-') {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`{}` starts with `-`, so it is an option and not a package name",
                name
            ),
        )
        .with_hint(
            "package names reach the manager's command line. If you meant to take a package \
             out of the set, a subtraction is `-name` at the start of its own line.",
        ));
    }
    Ok(())
}

/// Where a package line's options begin, if they do.
///
/// An `@` that **opens the name** is part of the name (Q23). npm's scoped packages are named
/// `@scope/name` — `@angular/cli`, `@bazel/bazelisk` — and `npm ls -g` prints them, so a name
/// Shall lists has to be a name Shall can be given back. Before this, `npm:@bazel/bazelisk` was
/// read as an empty name followed by an option list and refused with *"is not a list of
/// `key=value` options"*, which is baffling advice about a line nobody wrote wrongly.
///
/// Only the first character of the name is special. Every later `@` still opens the options,
/// which is what keeps `npm:@scope/name@version=1.2` a pinned scoped package rather than a
/// package called `@scope/name@version=1.2`.
fn option_separator(text: &str) -> Option<usize> {
    // Where the name starts: after the backend prefix and any space that follows it.
    let after_prefix = text.find(':').map(|i| i + 1).unwrap_or(0);
    let name_start = text[after_prefix..]
        .find(|c: char| !c.is_whitespace())
        .map(|off| after_prefix + off)
        .unwrap_or(after_prefix);
    // A quoted name is opaque: everything up to the closing quote is the name, `@` included.
    // Without this, `winget:"Some App@2"` would split inside the quotes and leave an unbalanced
    // name and an option list made of the rest of somebody's package.
    match quoted_span(text, name_start) {
        // The options begin *at* the character after the closing quote, so the `@` that opens
        // them sits exactly on that index — the one place a strict `>` silently loses it.
        Some(end) => text[end..].find('@').map(|off| end + off),
        None => text
            .char_indices()
            .find(|(i, c)| *c == '@' && *i != name_start)
            .map(|(i, _)| i),
    }
}

/// The index just past the closing quote, when `at` opens a quoted name.
///
/// `None` when there is no quote there or the quote is never closed — an unterminated quote is
/// left to the name parser, which has the origin needed to say so in a way the user can act on.
fn quoted_span(text: &str, at: usize) -> Option<usize> {
    let rest = text.get(at..)?;
    if !rest.starts_with('"') {
        return None;
    }
    rest[1..].find('"').map(|close| at + close + 2)
}

fn parse_package(origin: &Origin, text: &str, backends: &dyn BackendNames) -> Result<PackageDecl> {
    let (head, options) = match option_separator(text) {
        Some(at) => (text[..at].trim(), parse_short(origin, &text[at + 1..])?),
        None => (text, Options::default()),
    };

    if head.is_empty() {
        return Err(GrammarError::new(origin.clone(), "no package name"));
    }

    // Checked before the backend split so `re:^fonts-` gets the error that says what is
    // missing, rather than "`re` is not a backend" — which is true but useless.
    if let Some(pattern) = head.strip_prefix("re:") {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`re:{}` does not say which backend to match in",
                pattern.trim()
            ),
        )
        .with_hint("write `apt:re:^fonts-`. A pattern has to be matched somewhere."));
    }

    let (backend, candidates, rest) = match head.split_once(':') {
        Some((prefix, rest)) => {
            let (backend, candidates) = parse_prefix(origin, prefix, backends)?;
            (backend, candidates, rest.trim())
        }
        None => (None, Candidates::Priority, head),
    };

    let selector = match rest.strip_prefix("re:") {
        Some(pattern) => {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                return Err(GrammarError::new(origin.clone(), "`re:` has no pattern"));
            }
            // A pattern is matched against one manager's catalogue and frozen in that
            // manager's regex lock. Spread over a chain there is no single catalogue to
            // match and no single lock to write, so the line has to pin.
            if backend.is_none() {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`{}` must match in exactly one backend", head),
                )
                .with_hint(
                    "write `apt:re:^fonts-`. A pattern is matched against one manager's \
                     catalogue, so a chain has nothing to match against.",
                ));
            }
            Selector::Regex(pattern.to_string())
        }
        None => {
            if rest.is_empty() {
                return Err(GrammarError::new(
                    origin.clone(),
                    "no package name after the backend",
                ));
            }
            // A quoted name is taken verbatim, spaces and all. `winget list` answers with
            // `ARP\Machine\X64\Mozilla Firefox` — the identifier `winget install` takes back —
            // and a name Shall lists has to be a name Shall can be given (V.113). Quoting is
            // what keeps that from re-opening VI.1: prose is not quoted, so a typo is still an
            // error rather than a package named after itself.
            let name = match rest.strip_prefix('"') {
                Some(inner) => match inner.strip_suffix('"') {
                    Some(name) => {
                        if name.trim().is_empty() {
                            return Err(GrammarError::new(origin.clone(), "`\"\"` names nothing"));
                        }
                        // The quotes carry exactly what is between them, so an edge nobody can
                        // see is a second package that reads as the first: `"Firefox "` and
                        // `Firefox` are one thing declared twice, and every message about
                        // either of them prints the same word.
                        if name.trim() != name {
                            return Err(GrammarError::new(
                                origin.clone(),
                                format!("`{}` has a space at the edge of the name", rest),
                            )
                            .with_hint(format!(
                                "a quoted name is exact, so this is not the same package as \
                                 `\"{}\"` — which is almost certainly the one you mean.",
                                name.trim()
                            )));
                        }
                        if name.contains('"') {
                            return Err(GrammarError::new(
                                origin.clone(),
                                format!("`{}` has a quote inside a quoted name", rest),
                            )
                            .with_hint(
                                "a quoted name runs to the next quote; there is no escape for \
                                 one inside it.",
                            ));
                        }
                        // A manifest is a line per declaration. A name carrying a newline
                        // round-trips through this parser and then writes two lines — which is
                        // the wedged-config bug quoting exists to end, re-entering by the door
                        // quoting opened.
                        if name.chars().any(char::is_control) {
                            return Err(GrammarError::new(
                                origin.clone(),
                                "a quoted name contains a control character".to_string(),
                            )
                            .with_hint(
                                "a declaration is one line; a name that carries a newline or a \
                                 tab cannot be written as one.",
                            ));
                        }
                        name
                    }
                    // Distinguished, because "you never closed the quote" is unhelpful advice
                    // about a line that closed it and then kept going.
                    None => {
                        return Err(match inner.find('"') {
                            Some(close) => GrammarError::new(
                                origin.clone(),
                                format!("`{}` has more after the closing quote", rest),
                            )
                            .with_hint(format!(
                                "the name ends at the quote — `{}`. Options go after it, as \
                                 `@key=value`.",
                                &inner[..close]
                            )),
                            None => GrammarError::new(
                                origin.clone(),
                                format!("`{}` opens a quote and never closes it", rest),
                            )
                            .with_hint("write `winget:\"ARP\\Machine\\X64\\Mozilla Firefox\"`."),
                        });
                    }
                },
                // A package name is one word. Without this, any unrecognised prose becomes a
                // package literally named after itself — VI.1's "any typo becomes a package
                // name", which is what II.2's "an unrecognised line is an error" forbids.
                None => {
                    if rest.split_whitespace().count() > 1 {
                        return Err(GrammarError::new(
                            origin.clone(),
                            format!("`{}` is not a package name", rest),
                        )
                        .with_hint(
                            "a name with spaces in it has to be quoted: \
                             `winget:\"Mozilla Firefox\"`.",
                        ));
                    }
                    rest
                }
            };
            reject_leading_dash(origin, name)?;
            Selector::Name(name.to_string())
        }
    };

    Ok(PackageDecl {
        backend,
        candidates,
        selector,
        options,
    })
}

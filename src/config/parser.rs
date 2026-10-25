use crate::core::{Error, Result};
use crate::model::vars::Value;

/// Facts about the host used to evaluate `when` conditionals, so a single shared repo can
/// serve a heterogeneous fleet (Linux + macOS + Windows). The model reads these when it
/// resolves `when` gates in `active`, profiles and modules (II.2).
#[derive(Debug, Clone)]
pub struct HostFacts {
    pub os: String,
    pub arch: String,
    pub host: String,
    /// The distribution family: `debian`, `fedora`, `arch`, `suse`, … On a system with no
    /// distributions this is the OS name, so `family` always answers something.
    ///
    /// `os` already answers linux-or-windows, which is why this does not.
    pub family: String,
    /// The running user's home directory, when the machine can say it. `None` where no home
    /// is detectable — a `$home` in a value and a `when home ==` are then loud errors naming
    /// the fact, never an empty string that makes a path silently relative.
    pub home: Option<String>,
    /// The running user's login name, from the environment (`USER`, else `USERNAME`). `None`
    /// where neither answers, for the same reason as `home`: a fact that cannot be detected
    /// is an error, not an empty substitution.
    pub user: Option<String>,
    /// The resolved `vars` (Part IX), reached as `$name`. Empty until a caller supplies them,
    /// so a `when $role == …` in a repo with no `vars` file is an unknown key and says so.
    pub vars: crate::model::vars::Vars,
}

/// Detected once. `distro_family` reads `/etc/os-release` and `get_hostname` asks the OS; the
/// machine does not change its architecture partway through a command, and this is called on
/// every one of the resolver's entry points.
static DETECTED: once_cell::sync::Lazy<HostFacts> = once_cell::sync::Lazy::new(|| HostFacts {
    os: std::env::consts::OS.to_string(),
    arch: std::env::consts::ARCH.to_string(),
    host: crate::config::Config::get_hostname(),
    family: distro_family().unwrap_or_else(|| std::env::consts::OS.to_string()),
    home: detect_home(),
    user: detect_user(),
    vars: Default::default(),
});

impl HostFacts {
    /// Gather this machine's facts.
    pub fn current() -> Self {
        DETECTED.clone()
    }

    /// A detected fact's value, for value interpolation (`vars::expand`) as well as `when`.
    /// `$`-prefixed keys read the variables; bare keys read the machine. `home` and `user`
    /// answer `None` where the machine could not detect them, and the callers turn that into
    /// a loud error naming the fact — never an empty string.
    pub(crate) fn value_for(&self, key: &str) -> Option<Value> {
        // IX.4: `$name` is a variable you decided, `name` is a fact the machine reported. The
        // sigil is what lets Shall add a detected fact without changing the meaning of a file
        // where someone happened to use that word as a variable.
        if let Some(var) = key.strip_prefix('$') {
            return self.vars.get(var).cloned();
        }
        // A detected fact is always a string (W2), so `$var == family` follows the same
        // no-coercion equality as any other comparison.
        match key {
            "os" => Some(Value::Str(self.os.clone())),
            "arch" => Some(Value::Str(self.arch.clone())),
            "host" | "hostname" => Some(Value::Str(self.host.clone())),
            "family" => Some(Value::Str(self.family.clone())),
            "home" => self.home.clone().map(Value::Str),
            "user" => self.user.clone().map(Value::Str),
            _ => None,
        }
    }

    /// The resolved variables this run evaluates `$name` against.
    ///
    /// Resolved once per invocation and carried, never recomputed: a provider may read the
    /// clock or shell out, so a second resolution can disagree with the first, and a `plan`
    /// that disagrees with the `sync` executing it is not a plan (IX.6).
    pub fn with_vars(mut self, vars: crate::model::vars::Vars) -> Self {
        self.vars = vars;
        self
    }
}

/// The running user's home directory, as the `~` in a `link:` target already reads it.
/// `dirs` answers from the environment first and the user database second, so this agrees
/// with `resolve_target` by construction rather than by luck — a `$home` in content and a
/// `~/` in a target name the same directory. `None` where neither answers.
fn detect_home() -> Option<String> {
    dirs::home_dir()?.to_str().map(str::to_owned)
}

/// The running user's login name, from the environment. `USER` on Unix, `USERNAME` on
/// Windows; an empty answer counts as no answer, because an empty `$user` substituted into
/// a path is the silent relative-path failure `detect_home`'s `None` exists to prevent.
/// There is no user-database lookup here: `dirs` has no such function, and a new dependency
/// for one string the environment already carries is not justified.
fn detect_user() -> Option<String> {
    for key in ["USER", "USERNAME"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// The distribution family — `debian`, `fedora`, `arch`, `suse`, … — read from
/// `/etc/os-release`, which is the only place that answers it. `None` where there are no
/// distributions to tell apart.
pub fn distro_family() -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let text = std::fs::read_to_string("/etc/os-release").ok()?;
    parse_os_release_family(&text)
}

/// `ID_LIKE` names the family and `ID` names the distribution, so a derivative
/// (`linuxmint`, whose `ID_LIKE` is `ubuntu debian`) resolves to the family that decides
/// which artifact installs. `ID_LIKE` is checked first for exactly that reason.
/// **`pub` for the integration tests' sake.** The parser mirror links the library without
/// `cfg(test)` and must exercise the exact os-release family parser.
pub fn parse_os_release_family(text: &str) -> Option<String> {
    let field = |key: &str| -> Option<String> {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
            .map(|v| v.trim().trim_matches('"').to_lowercase())
    };

    const FAMILIES: [&str; 6] = ["debian", "fedora", "rhel", "suse", "arch", "alpine"];
    let id_like = field("ID_LIKE").unwrap_or_default();
    for family in FAMILIES {
        if id_like.split_whitespace().any(|w| w == family) {
            return Some(family.to_string());
        }
    }

    let id = field("ID")?;
    // `ubuntu` has no `ID_LIKE` on some releases and is not itself in the family list.
    match id.as_str() {
        "ubuntu" | "debian" | "raspbian" => Some("debian".into()),
        "fedora" => Some("fedora".into()),
        "centos" | "rhel" | "rocky" | "almalinux" => Some("rhel".into()),
        "opensuse" | "sles" => Some("suse".into()),
        "arch" | "manjaro" | "endeavouros" => Some("arch".into()),
        "alpine" => Some("alpine".into()),
        other => Some(other.to_string()),
    }
}

/// Evaluate a `when` predicate against host facts. Forms: `os == linux`, `arch != x86_64`,
/// `$count > 3`, `os in [linux, macos]`. Values are typed (W2): comparison is between values of
/// the same type, ordering is numbers only, and there is no truthiness — a bare `$flag` is an
/// error, not a non-empty test. Pure — unit tested.
pub fn eval_when(pred: &str, facts: &HostFacts) -> Result<bool> {
    let pred = pred.trim();

    // Membership form: `key in [a, b, c]`  (brackets optional), or `key in $listvar`.
    if let Some((key, rest)) = pred.split_once(" in ") {
        let actual = lhs_value(key.trim(), facts)?;
        let elements = list_operand(rest.trim(), facts)?;
        return Ok(elements.iter().any(|el| actual.equals(el)));
    }

    // Comparison form. Two-char operators are matched before one-char ones so `<=` is not read
    // as `<`; every operator is distinct so order among the two-char set does not matter.
    for op in ["==", "!=", "<=", ">="] {
        if let Some((l, r)) = pred.split_once(op) {
            return compare(l.trim(), op, r.trim(), facts);
        }
    }
    for op in ["<", ">"] {
        if let Some((l, r)) = pred.split_once(op) {
            return compare(l.trim(), op, r.trim(), facts);
        }
    }

    // No operator. A bare `$flag` is the truthiness trap W3 forbids: `false` is a non-empty
    // string, so `when $gpu` would fire on `gpu = false`. Refuse it, and say what to write.
    if let Some(var) = pred.strip_prefix('$') {
        return Err(Error::Config(format!(
            "`{}` is not a condition on its own — there is no truthiness; write `${} == true` or another comparison",
            pred, var
        )));
    }
    Err(Error::Config(format!(
        "invalid `when` predicate '{}' (use `key == value`, `key != value`, `key < number`, or `key in [..]`)",
        pred
    )))
}

/// The left side of a comparison is always a key — a detected fact or a `$variable` — and an
/// unknown one is an error, never a silent false (a typo'd `$rle` that read as false is a block
/// that never fires and never complains, IX.3).
fn lhs_value(key: &str, facts: &HostFacts) -> Result<Value> {
    if let Some(v) = facts.value_for(key) {
        return Ok(v);
    }
    // A fact this machine cannot answer is a different failure from a key that is not a
    // fact at all: the first names what could not be detected, the second is a typo.
    if ["home", "user"].contains(&key) {
        return Err(Error::Config(format!(
            "this machine cannot say `{}`, so `when {} == …` has nothing to compare",
            key, key
        )));
    }
    Err(Error::Config(format!("unknown `when` key '{}'", key)))
}

/// The right side is a value: another `$variable`, or a literal read with the same rules a `vars`
/// line uses, so `$gpu == true` and `gpu = true` mean the same thing.
fn rhs_value(token: &str, facts: &HostFacts) -> Result<Value> {
    if token.starts_with('$') {
        return lhs_value(token, facts);
    }
    Ok(Value::parse_literal(token))
}

fn compare(left: &str, op: &str, right: &str, facts: &HostFacts) -> Result<bool> {
    let a = lhs_value(left, facts)?;
    let b = rhs_value(right, facts)?;
    match op {
        "==" => Ok(a.equals(&b)),
        "!=" => Ok(!a.equals(&b)),
        _ => {
            let ord = a.order(&b).ok_or_else(|| {
                Error::Config(format!(
                    "`{} {} {}` compares a {} with a {}; ordering is only defined between numbers",
                    left,
                    op,
                    right,
                    a.type_name(),
                    b.type_name()
                ))
            })?;
            Ok(match op {
                "<" => ord.is_lt(),
                ">" => ord.is_gt(),
                "<=" => ord.is_le(),
                ">=" => ord.is_ge(),
                _ => unreachable!("operator set is closed"),
            })
        }
    }
}

/// The right side of `in`: a bracketed literal list, or a `$variable` that must itself be a list.
fn list_operand(rest: &str, facts: &HostFacts) -> Result<Vec<Value>> {
    if rest.starts_with('$') {
        return match lhs_value(rest, facts)? {
            Value::List(items) => Ok(items),
            other => Err(Error::Config(format!(
                "`{}` is a {}, not a list; `in` needs a list on the right",
                rest,
                other.type_name()
            ))),
        };
    }
    let inner = rest.trim_start_matches('[').trim_end_matches(']');
    Ok(inner
        .split(',')
        .map(|s| Value::parse_literal(s.trim()))
        .collect())
}

/// Split a removal target like `backend:name[@opts]` into `(Some(backend), bare_name)`
/// when the prefix names a real backend, or `(None, name)` otherwise. `@options` are
/// stripped from the name. `is_known_backend` decides whether a `prefix:` is a backend
/// (so package names that legitimately contain a colon aren't misread as `backend:name`).
///
/// This is the parsing `remove` must use to match how `install` reads its arguments —
/// passing the whole `backend:name` string to a backend's `info()`/`remove()` (which
/// expect the *bare* name) silently makes `remove backend:pkg` a no-op.
///
/// **It is the grammar's answer, because there is one parser.** This used to split on `:` and
/// then take `name_part.split('@').next()`, which had never heard of two rules the read side
/// learned separately:
///
/// - **`Q23`** — an `@` that *opens* the name is part of the name. `split_removal_target(
///   "npm:@angular/cli")` returned `(Some("npm"), "")`, so the seven call sites below —
///   `rebuild`, `cleanup` three times, `packages`, `upgrade` — carried an empty package name
///   into a removal. `Q23` was fixed on the read side and its sibling on the **remove** side
///   stayed live.
/// - **`V.113`** — a quoted name is opaque, spaces and `@` included. `winget:"Some App@2"`
///   split inside the quotes and produced half a name.
///
/// Neither is fixable by adding a case here; both are already right in
/// `grammar::statement::parse`, which is the only thing that knows where a name ends. A parse
/// failure falls back to `(None, input)` — the same answer this gave for an unrecognised
/// prefix — so the function stays infallible for a caller holding a raw CLI argument.
pub fn split_removal_target(
    input: &str,
    is_known_backend: impl Fn(&str) -> bool,
) -> (Option<String>, String) {
    use crate::config::grammar::{statement, Origin, Selector, Statement};

    let origin = Origin::new(std::path::PathBuf::from("<argument>"), 0);
    match statement::parse(&origin, input, &is_known_backend) {
        Ok(Statement::Package(decl)) | Ok(Statement::Absent(decl)) => match decl.selector {
            Selector::Name(name) => (decl.backend, name),
            // A pattern is not a removal target: it names a set, and the caller asked for one
            // package. Handing back the pattern text keeps the old shape — nothing matches, and
            // the caller says so — rather than inventing a name that was never written.
            Selector::Regex(pattern) => (decl.backend, pattern),
        },
        _ => (None, input.to_string()),
    }
}

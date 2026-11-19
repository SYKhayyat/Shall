//! What a parsed line must satisfy before it is a declaration: II.2's option table,
//! the keys each kind of resource accepts, and the pairs that only make sense together.

use super::*;

/// Every option rule in II.2, for every statement that carries options.
///
/// This runs on the finished statement rather than inside the header parse, because a block
/// body's keys are merged in after the header is parsed. Validating at the header let
/// `apt:jq@hold { version = 1.6 }` through — the same contradiction the short form refuses,
/// silent — and II.2 closes with the reason that cannot stand: silently ignoring an option
/// the user wrote is how a config grows lines that do nothing.
pub fn validate(origin: &Origin, stmt: &Statement) -> Result<()> {
    match stmt {
        Statement::Package(decl) => validate_options(origin, decl, false),
        Statement::Absent(decl) => validate_options(origin, decl, true),
        Statement::Shim(name, o) => validate_extra_options(origin, OptionKind::Shim, name, o, None),
        Statement::Service(name, o) => {
            validate_extra_options(origin, OptionKind::Service, name, o, None)
        }
        Statement::Link(name, o) => validate_link(origin, name, o),
        Statement::Schedule(name, o) => {
            validate_extra_options(origin, OptionKind::Schedule, name, o, None)
        }
        Statement::Setting(name, o) => validate_setting(origin, name, o),
        Statement::Exec(name, o) => crate::config::grammar::exec::validate_exec(origin, name, o),
        Statement::Generate(name, o) => {
            crate::config::grammar::exec::validate_generate(origin, name, o)
        }
        Statement::Dotfiles(name, o) => {
            validate_extra_options(origin, OptionKind::Dotfiles, name, o, None)
        }
        Statement::Firewall(name, o) => validate_firewall(origin, name, o),
        Statement::Dir(name, o) => validate_extra_options(origin, OptionKind::Dir, name, o, None),
        // The shape of the three options is checked by the same parser that later reads them,
        // so `check` can never meet a `@max-age` the grammar was willing to keep and it is not.
        Statement::Invariant(name, o) => {
            validate_extra_options(origin, OptionKind::Invariant, name, o, None)?;
            crate::model::invariant::Invariant::parse(origin, name, o).map(|_| ())
        }
        Statement::Repo { .. }
        | Statement::Use(..)
        | Statement::Param { .. }
        | Statement::Exclude(_)
        | Statement::Intersect(_)
        | Statement::Subtract(_)
        | Statement::Var { .. }
        | Statement::Expr(_) => Ok(()),
    }
}

/// The options each non-package statement understands (II.2's table).
///
/// A `schedule:` also needs `cron` and `run` to be *present*, which `model::schedule` checks
/// when it builds the job — that is a question about one line's meaning, not about which
/// words are legal, and it has an error that can name what is missing.
/// `scope` is on exactly the three statements where "for me" and "for the machine" can differ
/// (U19). A `service:` is the init system's business and a `repo:` is the manager's, so
/// neither takes it — a key that means nothing on a statement is a key that will be written
/// there and silently ignored.
/// A statement kind that carries `@options`, as the option tables are keyed.
///
/// An enum and not the `&str` from [`Statement::kind`]: the lookup below used to match on the
/// string with `_ => SCHEDULE_OPTION_KEYS` as its fall-through, so a kind added without a table
/// silently inherited schedule's — `@cron` accepted on something that has no schedule, its own
/// options refused, and no complaint from anywhere. The compiler asks the question now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Shim,
    Service,
    Link,
    Schedule,
    Setting,
    Exec,
    Generate,
    Dotfiles,
    Firewall,
    Dir,
    Invariant,
}

impl OptionKind {
    /// The keyword as written, for the refusal that names it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shim => "shim",
            Self::Service => "service",
            Self::Link => "link",
            Self::Schedule => "schedule",
            Self::Setting => "setting",
            Self::Exec => "exec",
            Self::Generate => "generate",
            Self::Dotfiles => "dotfiles",
            Self::Firewall => "firewall",
            Self::Dir => "dir",
            Self::Invariant => "invariant",
        }
    }

    /// Every kind that carries options — what a test quantifies over, so a kind added without
    /// a case in one of them is a failure and not an omission.
    pub const ALL: &'static [Self] = &[
        Self::Shim,
        Self::Service,
        Self::Link,
        Self::Schedule,
        Self::Setting,
        Self::Exec,
        Self::Generate,
        Self::Dotfiles,
        Self::Firewall,
        Self::Dir,
        Self::Invariant,
    ];
}

pub const SHIM_OPTION_KEYS: &[&str] = &["source", "scope", "user"];
pub const SERVICE_OPTION_KEYS: &[&str] = &["enabled", "status", "user"];
pub const LINK_OPTION_KEYS: &[&str] = &[
    "target", "content", "template", "decrypt", "identity", "scope", "backup", "user", "owner",
];
/// `enabled`, `persistent`, `jitter` and `elevated` are here even though no scheduler expresses
/// all four: the grammar says what a schedule may *say*, and `app/scheduler` says what each OS
/// can *keep*. Refusing an option in the parser because one platform cannot hold it would make
/// a portable model file unreadable on the machine it was written for.
pub const SCHEDULE_OPTION_KEYS: &[&str] = &[
    "cron",
    "run",
    "notify",
    "enabled",
    "persistent",
    "jitter",
    "elevated",
    "user",
];
pub const SETTING_OPTION_KEYS: &[&str] = &["value", "scope", "user"];
/// `target` is where the tree is mirrored to; absent means the home directory, which is what a
/// dotfiles tree mirrors by definition. There is deliberately no per-file option: the tree has
/// no place to write one, which is why it never decrypts (U24).
pub const DOTFILES_OPTION_KEYS: &[&str] = &["target"];
/// `value` is the policy a `default/...` rule sets (`allow` or `deny`). A port rule takes no
/// options: `firewall:22/tcp` is the whole declaration.
pub const FIREWALL_OPTION_KEYS: &[&str] = &["value"];
pub const DIR_OPTION_KEYS: &[&str] = &["user", "owner", "mode"];
/// The three things an invariant says. `backed-by` is `ADAPTER:TARGET` in one value rather than
/// two keys, because the pair is one statement of where the copy lives — split in two, a reader
/// could write half of it and the line would still parse.
pub const INVARIANT_OPTION_KEYS: &[&str] = &["path", "backed-by", "max-age"];

pub const USER_SCOPABLE_KINDS: &[ResourceKind] = &[
    ResourceKind::Link,
    ResourceKind::Dir,
    ResourceKind::Shim,
    ResourceKind::Setting,
    ResourceKind::Service,
    ResourceKind::Schedule,
];

pub fn set_user_scope(origin: &Origin, stmt: &mut Statement, user: &str) -> Result<()> {
    let kind = stmt.kind();
    if !kind.is_some_and(|kind| USER_SCOPABLE_KINDS.contains(&kind)) {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`use user:NAME {{ ... }}` does not support `{}` statements",
                stmt.kind().map_or("this", ResourceKind::as_str)
            ),
        ));
    }
    let options = match stmt {
        Statement::Link(_, options)
        | Statement::Dir(_, options)
        | Statement::Shim(_, options)
        | Statement::Setting(_, options)
        | Statement::Service(_, options)
        | Statement::Schedule(_, options) => options,
        _ => {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`use user:NAME {{ ... }}` does not support `{}` statements",
                    stmt.kind().map_or("this", ResourceKind::as_str)
                ),
            ))
        }
    };
    if !options.contains("user") {
        options.set("user", user);
    }
    Ok(())
}

/// The option keys that answer to ONE value, across every kind.
///
/// A key given twice is a list (II.2) for the keys that mean it — `@requires=`, `@after=` —
/// and a silent first-wins demotion for every other. This is the set of the others: a repeat
/// is refused at [`validate_extra_options`] and in the package path below instead of being
/// demoted by `one()`'s `.first()`. Deliberately absent: `requires`, `after` (lists by
/// design), `on` and `notify` (comma-bearing values whose multiplicity lives in the value).
pub(crate) const SINGLE_VALUE_OPTION_KEYS: &[&str] = &[
    EXPORT_KEY,
    "source",
    "enabled",
    "status",
    "target",
    "content",
    "template",
    "decrypt",
    "identity",
    "backup",
    "cron",
    "undo",
    "value",
    "scope",
    "user",
    "owner",
    "mode",
    "path",
    "backed-by",
    "max-age",
    "version",
    "hold",
    "expires",
    "until",
    "sha256",
    "bin",
    "channel",
    "asset",
    "download_only",
    "runs",
];

/// A `link:` names one mode — inline `@content=`, a `@template=true` source file, a
/// `@decrypt=` source, or a plain symlink — and the installer reads them in that order.
/// A line naming two is a line whose second mode nobody reads: `content` wins in the
/// installer while `check` used to call the same line unverifiable, so the refusal lives
/// here, where the line is written, rather than in either reader. `@template=` is a flag
/// with a value, so anything but `true` or `false` is refused rather than read as a plain
/// symlink that happens to carry a word nobody acts on.
pub(crate) fn validate_link(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Link, name, options, None)?;
    if let Some(t) = options.one("template") {
        if t != "true" && t != "false" {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`link:{}` has `@template={}`, which is neither on nor off",
                    name, t
                ),
            )
            .with_hint(
                "write `@template` or `@template=true` to render, or drop it for a plain link.",
            ));
        }
    }
    // `@template=false` states the default and is not a mode, so it never collides:
    // only a `true` template competes with content and decrypt.
    let mut modes = Vec::new();
    if options.contains("content") {
        modes.push("content");
    }
    if options.contains("decrypt") {
        modes.push("decrypt");
    }
    if options.one("template") == Some("true") {
        modes.push("template");
    }
    if modes.len() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`link:{}` names two modes (`@{}` and `@{}`), and only the first one runs",
                name, modes[0], modes[1]
            ),
        )
        .with_hint("a `link:` is inline `@content=`, a `@template=true` source, a `@decrypt=` source, or a plain link — pick one."));
    }
    Ok(())
}

/// A firewall line names a rule the grammar can read, and a default policy says which one.
pub(crate) fn validate_firewall(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Firewall, name, options, None)?;
    let rule = crate::model::firewall::Rule::parse(name)
        .map_err(|e| GrammarError::new(origin.clone(), e))?;
    match rule {
        crate::model::firewall::Rule::Default { .. } => match options.one("value").map(str::trim) {
            Some("allow") | Some("deny") => Ok(()),
            _ => Err(GrammarError::new(
                origin.clone(),
                format!("`firewall:{}` needs a policy", name),
            )
            .with_hint(
                "say which way it goes: `@value=deny` or `@value=allow`. A default policy \
                     with no value declares nothing, and it is the most consequential line in \
                     a firewall.",
            )),
        },
        // A port rule is its own declaration; `@value=` on one would be a second way to say
        // the same thing, and a confusing one (`firewall:22/tcp @value=deny` reads as both).
        crate::model::firewall::Rule::Port { .. } => match options.one("value") {
            None => Ok(()),
            Some(_) => Err(GrammarError::new(
                origin.clone(),
                format!("`firewall:{}` takes no `value`", name),
            )
            .with_hint(
                "a declared port is open — that is what declaring it means. To close one, \
                 delete the line; `@value=` belongs on `default/incoming` only.",
            )),
        },
    }
}

/// [`keys_for`] for a test in another binary: the table a kind reads, so a kind wired to the
/// wrong one — which still compiles — can be caught by quantifying over [`OptionKind::ALL`].
pub fn keys_for_kind(kind: OptionKind) -> &'static [&'static str] {
    keys_for(kind)
}

/// The options one kind may carry. **Exhaustive, with no default arm** — a tenth kind does not
/// compile until it says which options it takes.
pub(crate) fn keys_for(kind: OptionKind) -> &'static [&'static str] {
    match kind {
        OptionKind::Shim => SHIM_OPTION_KEYS,
        OptionKind::Service => SERVICE_OPTION_KEYS,
        OptionKind::Link => LINK_OPTION_KEYS,
        OptionKind::Schedule => SCHEDULE_OPTION_KEYS,
        OptionKind::Setting => SETTING_OPTION_KEYS,
        OptionKind::Exec => crate::config::grammar::exec::EXEC_OPTION_KEYS,
        OptionKind::Generate => crate::config::grammar::exec::GENERATE_OPTION_KEYS,
        OptionKind::Dotfiles => DOTFILES_OPTION_KEYS,
        OptionKind::Firewall => FIREWALL_OPTION_KEYS,
        OptionKind::Dir => DIR_OPTION_KEYS,
        OptionKind::Invariant => INVARIANT_OPTION_KEYS,
    }
}

/// An `exec:` names a script and, optionally, how many times its content may run. The name
/// must be non-empty; `runs`, if present, is a positive count or the word `always`.
/// `@scope=` must name one of the two things it can mean. A misspelling that parsed as
/// "default" would be a line that reads as a decision and behaves as if nobody made one.
pub(crate) fn validate_scope(
    origin: &Origin,
    prefix: &str,
    name: &str,
    options: &Options,
) -> Result<()> {
    let Some(written) = options.one("scope") else {
        return Ok(());
    };
    if crate::model::scope::Scope::parse(written).is_none() {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{}:{}` has an invalid `scope={}`", prefix, name, written),
        )
        .with_hint(format!(
            "scope is {}. Omitting it means whatever this store does by default.",
            crate::model::scope::Scope::vocabulary()
        )));
    }
    Ok(())
}

/// Split `SCHEMA/KEY` into its halves. The one place the shape is decided, so the parser's
/// refusal and the adapter's lookup cannot disagree about what a setting names.
pub fn split_setting(name: &str) -> Option<(&str, &str)> {
    let (schema, key) = name.split_once('/')?;
    let (schema, key) = (schema.trim(), key.trim());
    if schema.is_empty() || key.is_empty() || key.contains('/') {
        return None;
    }
    Some((schema, key))
}

/// A setting names a schema, a key inside it, and the value it must hold. A line missing any
/// of the three describes no state, and applying it would mean choosing on the user's behalf
/// which key they meant.
pub(crate) fn validate_setting(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Setting, name, options, None)?;

    if split_setting(name).is_none() {
        return Err(
            GrammarError::new(origin.clone(), format!("`{}` is not `SCHEMA/KEY`", name)).with_hint(
                "a setting names the schema and the key inside it, separated by one `/`: \
             `setting:org.gnome.desktop.interface/color-scheme @value=prefer-dark`.",
            ),
        );
    }

    if options.one("value").is_none_or(str::is_empty) {
        return Err(
            GrammarError::new(origin.clone(), format!("`setting:{}` has no value", name))
                .with_hint(
                    "say what the key must hold: `@value=prefer-dark`. A setting with no value \
                     declares nothing.",
                ),
        );
    }
    // A repeated `@value=` is refused one gate up, in `validate_extra_options`, with every
    // other single-valued key.
    Ok(())
}

/// Refuse any option this kind does not take, then check `@scope=` if it took one.
///
/// `hint` is the kind's own sentence about what its options *mean*. The generic
/// "takes: a, b, c" lists the keys and explains none of them, and a refusal that only names
/// the legal spellings makes the reader go and look up what they do — so each caller keeps the
/// sentence it had, and only the *table* is shared.
pub fn validate_extra_options(
    origin: &Origin,
    kind: OptionKind,
    name: &str,
    options: &Options,
    hint: Option<&str>,
) -> Result<()> {
    let prefix = kind.as_str();
    let legal = keys_for(kind);
    validate_export(origin, options)?;
    for key in options.keys() {
        // `@export` is legal on every line that carries options (II.94): it is a statement about
        // the export, not about the resource, so it belongs to no kind's own list.
        if legal.contains(&key) || key == EXPORT_KEY {
            // A key that answers to ONE value refuses a second occurrence outright. `one()`
            // reads the first, so a repeated `@status=running,@status=stopped` kept the first
            // and did nothing — a hand-edit flip that parsed, exited 0, changed nothing.
            // Cardinality used to be checked for five keys one `if` at a time; this is the
            // same refusal for every single-valued key in the grammar, at the one gate every
            // typed line passes.
            if SINGLE_VALUE_OPTION_KEYS.contains(&key) && options.all(key).len() > 1 {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{key}` takes one value, given {}", options.all(key).len()),
                )
                .with_hint(
                    "`one()` of a repeated key reads the FIRST value, so the second was \
                     silently discarded. Write the one you mean.",
                ));
            }
            continue;
        }
        let what = if legal.is_empty() {
            format!(
                "`{}:{}` has an option `{}`, but it takes none",
                prefix, name, key
            )
        } else {
            format!("`@{}` is not an option on `{}:`", key, prefix)
        };
        let hint = match hint {
            Some(h) => h.to_string(),
            None => format!("`{}:{}` takes: {}.", prefix, name, legal.join(", ")),
        };
        return Err(GrammarError::new(origin.clone(), what).with_hint(hint));
    }
    validate_scope(origin, prefix, name, options)
}

/// The opt-out from an export (II.94), spelled once because three gates refuse a typo of it.
pub const EXPORT_KEY: &str = "export";

/// `@export=yes|no` — the closed vocabulary, and the reason it is closed.
///
/// **`@export=nope` must not read as the default.** An opt-out that a typo turns into "export it
/// anyway" is worse than no opt-out: the person believes a line is excluded, the artifact carries
/// it, and nothing anywhere says so. Two words, named in the refusal (`[declare] install`'s
/// three-word precedent, `#94`).
pub(crate) fn validate_export(origin: &Origin, options: &Options) -> Result<()> {
    let Some(written) = options.one(EXPORT_KEY) else {
        return Ok(());
    };
    if written == "yes" || written == "no" {
        return Ok(());
    }
    Err(GrammarError::new(
        origin.clone(),
        format!("`@export={}` is not a yes or a no", written),
    )
    .with_hint(
        "`@export=no` keeps the line out of `shall export` while `sync` still applies it. \
         Anything else would be an opt-out that silently did nothing.",
    ))
}

/// Every option a package line may carry (II.2's table). Hooks are `*_install`
/// (`after_install`, `before_install`, …), so they are matched by suffix rather than listed.
///
/// `until` is here and refused below unless the line is `absent:` — II.2 puts it on
/// `absent:` only, and "not an option" would be the wrong error for a key that exists.
pub(crate) const PACKAGE_OPTION_KEYS: &[&str] = &[
    "version",
    "hold",
    // II.94's opt-out. Legal on every line, read by the exporter and by nothing else.
    EXPORT_KEY,
    "expires",
    "until",
    "requires",
    "sha256",
    "formats",
    "asset",
    "bin",
    "channel",
    "allow_http",
    "unverified",
    // Q49. "Write into the environment the OS owns" — legal only where a manager can be told
    // that, and refused by name everywhere else. `capability::OS_OWNED_ENV` is the one table.
    "system",
    "health",
    "download_only",
    // U39. Legal only on a backend that installs from something other than the name, and
    // refused by name everywhere else — `capability::INSTALLS_FROM_SOURCE` is the one table.
    "url",
    // A shim is a PATH stand-in that forwards to a managed tool. `@shim=true` asks for one on
    // the tool's own line — the form R3 named when it deleted the imperative command — and
    // `@sandbox=true` asks for the same shim and confines `shall run` as well. Both are read by
    // `sync`, and this table refused them until Q18, so the form the ruling pointed at was the
    // one form that did not parse.
    "shim",
    "sandbox",
    // Q18. The geometry of a declared storage object, and snap's confinement. Legal on the
    // backends that read them and refused by name elsewhere — `capability::SCOPED_OPTIONS` is
    // the one table, and the test beside it asserts every key there appears here.
    "size",
    "quota",
    "mount",
    "mount_options",
    "classic",
    // Q19. A declared `@size` is applied to a volume that already exists, and the one direction
    // that can destroy data says so on the line.
    "allow_shrink",
];

pub fn package_option_keys() -> &'static [&'static str] {
    PACKAGE_OPTION_KEYS
}

/// Whether `key` names an option a package line may carry.
///
/// The one predicate over II.2's table. It exists because the lexer needs the same answer the
/// validator needs: `@version=1.0.0@hold` has to be told apart from `@source=owner/repo@v2`, and
/// the only thing that distinguishes them is whether the text after the `@` names an option.
pub(crate) fn is_package_option_key(key: &str) -> bool {
    PACKAGE_OPTION_KEYS.contains(&key) || key.ends_with("_install")
}

/// Options that are only meaningful on some backends — one that resolves a name to several
/// downloadable artifacts, one that publishes version streams, one that installs from a URL,
/// one that carves up a disk. Each is refused by name on any other backend: an option nobody
/// reads is a line that does nothing.
/// Takes the backend and the options rather than a declaration, because the same rules apply
/// to a backend's options body in `priority` (VIII.2) and one of them had to be the caller.
pub fn validate_backend_options(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    // One function per option family, called in the order the refusals have to be asked: a
    // line is checked for options its backend cannot read at all (`@formats` on `apt`) before
    // the ones about their own values, and the checksum rules last because
    // `@asset=all,sha256=…` is refused by the more specific sentence first.
    validate_artifact_options(origin, backend, o)?;
    validate_install_source_options(origin, backend, o)?;
    validate_scoped_options(origin, backend, o)?;
    validate_paired_options(origin, o)?;
    validate_opt_outs(origin, backend, o)?;
    validate_checksum_options(origin, backend, o)
}

/// Which files of a release a line may choose between, and how it chooses them.
fn validate_artifact_options(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    use crate::backends::artifact::{AssetPattern, FormatOrder};
    use crate::backends::capability;

    for key in ["formats", "asset", "bin"] {
        if !o.contains(key) {
            continue;
        }
        // A line with no prefix is resolved through `priority` later, so the backend that will
        // answer it is not known here. Refusing would break `fd@formats=deb`; the resolver
        // enforces it once the backend is known.
        let Some(backend) = backend else { continue };
        if !capability::selects_artifacts(backend) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(format!(
                "`{}` picks between several files of one release. Backends that offer a \
                 choice: {}. Everywhere else the ecosystem already decided the file.",
                key,
                capability::artifact_backends()
            )));
        }
    }

    if o.contains("channel") {
        if let Some(backend) = backend {
            if !capability::has_channels(backend) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@channel` is not an option on `{}`", backend),
                )
                .with_hint(format!(
                    "a channel is a version stream, not a file. Backends that publish \
                     channels: {}.",
                    capability::channel_backends()
                )));
            }
        }
        if o.all("channel").len() > 1 {
            return Err(
                GrammarError::new(origin.clone(), "`@channel` takes one value").with_hint(
                    "there is no fallback across version streams — trying `edge` and settling for \
                 `stable` would silently downgrade the machine. Name the one you want.",
                ),
            );
        }
    }

    for name in o.all("formats") {
        FormatOrder::parse_all([name])
            .map_err(|e| GrammarError::new(origin.clone(), e.to_string()))?;
    }

    if let Some(pattern) = o.one("asset") {
        AssetPattern::parse(pattern)
            .map_err(|e| GrammarError::new(origin.clone(), e.to_string()))?;
    }

    // `@download_only` (D3b) means "fetch but do not install" — a distinction only a backend
    // that downloads a file can draw. Every other backend hands the whole job to a package
    // manager, so there is no fetch-without-install to ask for.
    if o.contains("download_only") {
        if let Some(backend) = backend {
            if !capability::downloads(backend) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@download_only` is not an option on `{}`", backend),
                )
                .with_hint(format!(
                    "it fetches a file without installing it, which only {} do.",
                    capability::download_backends()
                )));
            }
        }
    }
    Ok(())
}

/// U39's install source: a key only a manager that installs by one string and removes by another
/// can act on.
fn validate_install_source_options(
    origin: &Origin,
    backend: Option<&str>,
    o: &Options,
) -> Result<()> {
    use crate::backends::capability;

    // U39's install source. On a backend that installs by name it is a line that does nothing,
    // and — worse than the artifact options above — it would read as the *name* being wrong.
    for key in PACKAGE_OPTION_KEYS
        .iter()
        .filter(|k| capability::is_source_key(k))
    {
        if !o.contains(key) {
            continue;
        }
        let Some(backend) = backend else { continue };
        if capability::install_source_key(backend).is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(format!(
                "it says where to install a package from, for a manager that installs by one \
                 string and removes by another: {}. Everywhere else the name is both.",
                capability::source_backends(key)
            )));
        }
    }
    Ok(())
}

/// Q18's family: a key one kind of backend reads and no other can act on.
fn validate_scoped_options(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    use crate::backends::capability;

    // Q18's family: a key one kind of backend reads and no other can act on. Same shape as the
    // install source above and for the same reason — on `apt` a `@quota=` would be read as the
    // machine having been told something, when nothing anywhere would act on it.
    for key in PACKAGE_OPTION_KEYS
        .iter()
        .filter(|k| capability::is_scoped_option(k))
    {
        if !o.contains(key) {
            continue;
        }
        let Some(backend) = backend else { continue };
        if !capability::takes_scoped_option(backend, key) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(capability::scoped_option_reason(key)));
        }
    }
    Ok(())
}

/// Options that are only meaningful beside another option on the same line.
fn validate_paired_options(origin: &Origin, o: &Options) -> Result<()> {
    // `@mount_options` fills the option field of the fstab entry `@mount` writes. Without
    // `@mount` there is no entry, so the key is read by nothing — the same "line that does
    // nothing" the refusals above exist to prevent, one level in.
    if o.contains("mount_options") && !o.contains("mount") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@mount_options` has no `@mount` to apply to",
        )
        .with_hint(
            "it fills the option field of the fstab entry `@mount=` writes, so on its own it \
             reaches nothing: add `@mount=/where`, or drop it.",
        ));
    }

    // Q19's opt-out, and the same one-level-in rule. `@allow_shrink` permits a *declared* size to
    // take space back, so without `@size` there is no declaration for it to permit — and a line
    // that reads as "shrinking is allowed here" while nothing can shrink is worse than a line
    // that does nothing, because someone will believe it.
    if o.contains("allow_shrink") && !o.contains("size") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@allow_shrink` has no `@size` to apply to",
        )
        .with_hint(
            "it lets a smaller `@size=` take space back off an existing volume, so on its own \
             it permits nothing: add `@size=`, or drop it.",
        ));
    }
    Ok(())
}

/// SEC2's and D5's opt-outs: each relaxes a rule, and a rule only some backends have.
fn validate_opt_outs(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    use crate::backends::capability;

    // SEC2's two opt-outs each relax a rule, and the rules have different reach — which is why
    // they are checked apart and never as a pair. Plain HTTP is only a question where Shall
    // itself fetches a URL; a checksum is a question wherever *something* vouches for the
    // bytes, and a manager can be that something (Q5). On a backend that verifies nothing
    // either flag is a line that does nothing, which II.2 refuses.
    if o.contains("allow_http") {
        if let Some(backend) = backend.filter(|b| !capability::downloads(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@allow_http` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it relaxes a rule about downloading and running a file, which only {} do. \
                 Everywhere else the package manager chose the URL, not the declaration.",
                capability::download_backends()
            )));
        }
    }

    if o.contains("system") {
        if let Some(backend) = backend.filter(|b| !capability::accepts_system(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@system` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it says this line may install into an environment the operating system owns, \
                 which is only a decision where the OS claims one: {}. Everywhere else there is \
                 nothing to override.",
                capability::system_backends()
            )));
        }
    }

    if o.contains("unverified") {
        if let Some(backend) = backend.filter(|b| !capability::accepts_unverified(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@unverified` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it says nothing checked the bytes, which is only a decision where something \
                 would have: {}. Everywhere else the package manager's own signed index \
                 answers for them.",
                capability::unverified_backends()
            )));
        }
    }
    Ok(())
}

/// One hash, one file: what a checksum can and cannot vouch for.
fn validate_checksum_options(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    use crate::backends::capability;

    // `@asset=all` installs every match, so there is no single artifact for one hash to cover.
    // Checked before the pinned-format rule below: both objections are true of
    // `@asset=all,sha256=…`, and this one names the reason the line cannot be fixed by
    // pinning a format.
    if o.one("asset")
        .is_some_and(|a| a.eq_ignore_ascii_case("all"))
        && o.contains("sha256")
    {
        return Err(GrammarError::new(
            origin.clone(),
            "`@asset=all` and `@sha256=` cannot both be set",
        )
        .with_hint(
            "`all` installs several files and one hash cannot verify them. Pin one file, or \
             drop the checksum.",
        ));
    }

    // One hash cannot cover an asset that varies by machine (D6): a shared module says
    // `github:x/y` and the Debian box downloads the `.deb` while the Fedora box downloads the
    // `.rpm`. A hand-written hash is only a claim about a file when the line names one file,
    // so it is legal only where the format is pinned to exactly one. Everywhere else the hash
    // is generated content and lives in `locks/<backend>.toml`.
    if o.contains("sha256")
        && backend.is_some_and(capability::selects_artifacts)
        && o.all("formats").len() != 1
    {
        let said = o.all("formats").len();
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`@sha256` needs the line to pin exactly one format, and it {}",
                if said == 0 {
                    "pins none".to_string()
                } else {
                    format!("lists {}", said)
                }
            ),
        )
        .with_hint(
            "one release ships several files and one hash cannot verify them all. Add \
             `@formats=` naming one, or drop the checksum — Shall records the hash of what it \
             downloaded in `locks/` either way.",
        ));
    }
    Ok(())
}

/// Option rules from II.2's table that are about the options themselves rather than any
/// one backend.
pub(crate) fn validate_options(origin: &Origin, decl: &PackageDecl, absent: bool) -> Result<()> {
    let o = &decl.options;

    // II.2's table is the whole list. An unknown key used to be kept and handed downstream,
    // where something might act on it — `@lease=2h` is the one that mattered: II.16 retired
    // it, nothing writes it, and `StateRegistry::add` still read it and turned it into a
    // real expiry. So a key this document deleted was silently still a package that
    // uninstalls itself (S19). An option nobody reads is a line that does nothing; an
    // option someone still reads is worse.
    for key in o.keys() {
        if is_package_option_key(key) {
            // Same single-value refusal the typed lines get, one gate earlier than any
            // reader's `one()` could demote a repeat silently. (`@requires=` stays a list;
            // it is not in [`SINGLE_VALUE_OPTION_KEYS`].)
            if SINGLE_VALUE_OPTION_KEYS.contains(&key) && o.all(key).len() > 1 {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{key}` takes one value, given {}", o.all(key).len()),
                )
                .with_hint(
                    "a repeated single-value key keeps only its first — write the one you mean.",
                ));
            }
            continue;
        }
        let mut err = GrammarError::new(origin.clone(), format!("`@{}` is not an option", key));
        err = match key {
            // The one worth naming, because it used to work.
            "lease" | "duration" => err.with_hint(
                "a lease is a dated line now: `@expires=2026-07-17T14:00`. A file cannot hold \
                 \"2 hours\" — it would mean something different every time it was read.",
            ),
            _ => err.with_hint(format!(
                "options on a package are: {}, and the `*_install` hooks.",
                PACKAGE_OPTION_KEYS.join(", ")
            )),
        };
        return Err(err);
    }

    // A health check decides whether the machine is rolled back (XIII.5), so a line whose
    // check cannot be understood must not parse. `@health=port:donkey` that read as a shell
    // command would be a probe that fails every time and reverts every sync.
    validate_export(origin, o)?;

    if let Some(written) = o.one("health") {
        if crate::model::health::Probe::parse(written).is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@health={}` is not a check", written),
            )
            .with_hint(
                "a health check is `port:8080` — something must be listening — or a command \
                 that exits 0, written plainly or as `cmd:systemctl is-active nginx`.",
            ));
        }
    }

    // `@hold` says "never upgrade this"; `@version=` says "this exact version". Together
    // they are a contradiction, not a refinement: hold means whatever is installed, and
    // version means something specific that may not be it. **Judged by VALUE, not presence**:
    // every consumer reads what `hold` SAYS, so `@hold=false` — "do not hold" — beside a
    // version is a line that means one thing, and refusing it with advice about keeping
    // whatever is installed misread the line to its author.
    let holds = o.one("hold").map(|v| {
        !v.is_empty() && !v.eq_ignore_ascii_case("false") && !v.eq_ignore_ascii_case("no")
    });
    if holds == Some(true) && o.contains("version") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@hold` and `@version=` contradict each other",
        )
        .with_hint(
            "`@hold` keeps whatever is installed; `@version=` pins a specific one. Pick one.",
        ));
    }

    // `requires` is install ORDERING for things outside a package manager's own dependency
    // graph (V.29). A bare name would have to be resolved via `priority`, and the whole
    // point is that these are things with no one to ask.
    for req in o.all("requires") {
        if !req.contains(':') {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`requires = {}` is a bare name", req),
            )
            .with_hint("`requires` needs a backend: `requires = apt:libfoo`."));
        }
    }

    // II.2: `expires` and `until` are absolute datetimes. A duration cannot work in a file
    // — the machine reading it next week has no idea when you wrote it (V.38), which is
    // exactly why `@lease=2h` was inert.
    for key in ["expires", "until"] {
        if let Some(v) = o.one(key) {
            if !is_absolute_datetime(v) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{}={}` is not an absolute date and time", key, v),
                )
                .with_hint(
                    "write it out in full: `@expires=2026-07-17T14:00`. A duration cannot \
                     work in a file — whoever reads it later has no idea when you wrote it.",
                ));
            }
        }
    }

    // `until` is the mirror of `expires` and only makes sense on `absent:` (absent now,
    // present after). On a present line it would mean "install this later", which the
    // grammar has no way to act on — so it is refused there, naming the file and line,
    // rather than parsed and quietly ignored.
    if !absent && o.contains("until") {
        return Err(
            GrammarError::new(origin.clone(), "`@until` is only for `absent:` lines").with_hint(
                "`@until` lifts an `absent:` line on a date (absent now, present after). To make \
             a present line lapse on a date, use `@expires`.",
            ),
        );
    }

    validate_backend_options(origin, decl.backend.as_deref(), &decl.options)?;
    Ok(())
}

/// Accepts RFC3339 and the `YYYY-MM-DDTHH:MM` form II.2 uses in its example. Rejects
/// anything that reads as a duration.
fn is_absolute_datetime(v: &str) -> bool {
    if chrono::DateTime::parse_from_rfc3339(v).is_ok() {
        return true;
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d"] {
        if chrono::NaiveDateTime::parse_from_str(v, fmt).is_ok()
            || chrono::NaiveDate::parse_from_str(v, fmt).is_ok()
        {
            return true;
        }
    }
    false
}

/// The `absent:`-style prefixes, for building an "unrecognised line" message that lists
/// what was expected. Derived from [`KEYWORDS`] rather than listed again — the copy that
/// used to live here knew six of the eleven.
pub fn known_prefixes() -> Vec<&'static str> {
    KEYWORDS
        .iter()
        .filter(|k| k.takes_colon())
        .map(|k| k.spelling)
        .collect()
}

/// Every reserved word and what it is, for anything that has to quantify over the language.
///
/// The bare word, never the spelling: a caller asking "what words does this grammar reserve"
/// wants `link`, and one asking "what prefixes are there" wants [`known_prefixes`]. Two
/// questions, two functions, one table — so the answer cannot drift the way the three copies
/// this table replaced did.
pub fn reserved_words() -> Vec<(&'static str, KeywordRole)> {
    KEYWORDS.iter().map(|k| (k.word(), k.role)).collect()
}

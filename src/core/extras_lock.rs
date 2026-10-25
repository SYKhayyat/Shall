//! The applied-extras ledger (S20): what `sync` has actually put in place, so it can tell
//! when a `repo:` / `shim:` / `service:` / `link:` / `schedule:` line is *removed*.
//!
//! Packages have the registry: Shall records what it installed, so deleting a package line
//! makes the package drift and `sync` removes it. Extras had no such record — apply was
//! one-way. Delete a `service:nginx` line and nothing disabled the service; delete a `repo:`
//! line and the repository stayed configured. `sync` could not even *detect* the removal,
//! because it had nothing to compare "what is declared now" against.
//!
//! declared set against the recorded one; anything recorded-but-no-longer-declared is drift,
//! and gets undone — exactly what removing a package line already does.
//!

use crate::config::grammar::{ResourceKind, Statement};
use crate::core::ledger::LockFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The identity of one applied extra: **what kind of thing it is, and which one**.
///
/// `Statement::key()` produces two different key spaces and its type does not say which. A
/// package statement keys `backend:name`; a keyword statement keys `kind:subject`. The hazard is
/// named in `Statement::kind`'s own doc comment — *"re-splitting `key` on `:` … would read
/// `apt:jq` as the kind `apt`"* — and it was still being re-split, by hand, in three places that
/// deal in extras keys and by five more that deal in package keys and must not be confused with
/// them.
///
/// This is the extras half, as one type. It is **the only producer and the only reader** of a
/// `<kind>:<subject>` string: `Display` writes it, [`FromStr`](std::str::FromStr) reads it, and
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtraKey {
    pub kind: ResourceKind,
    /// Everything after the keyword, verbatim. A `repo:` subject is itself `backend:spec`, which
    /// is why this is not split further here: that inner structure is the repo backend's, and
    /// splitting it twice in one type is how the second reader gets it wrong.
    pub subject: String,
    pub user: Option<String>,
}

pub const ACCOUNT_SCOPED: &[ResourceKind] = &[
    ResourceKind::Shim,
    ResourceKind::Setting,
    ResourceKind::Service,
    ResourceKind::Schedule,
];

const USER_MARK: &str = "@user=";

fn account_name_is_well_formed(name: &str) -> bool {
    !name.is_empty()
        && !name.contains(['@', ':'])
        && !name.chars().any(|c| c.is_whitespace() || c.is_control())
}

fn split_account_suffix(row: &str) -> std::result::Result<(String, Option<String>), ()> {
    let Some(at) = row.find(USER_MARK) else {
        return Ok((row.to_string(), None));
    };
    let (subject, name) = row.split_at(at);
    let name = &name[USER_MARK.len()..];
    if subject.is_empty() || !account_name_is_well_formed(name) {
        return Err(());
    }
    Ok((subject.to_string(), Some(name.to_string())))
}

impl ExtraKey {
    pub fn new(kind: ResourceKind, subject: impl Into<String>) -> Self {
        let subject = subject.into();
        debug_assert!(
            Self::subject_is_recordable(kind, &subject),
            "an account-scoped subject carrying the account marker cannot round-trip: {subject}"
        );
        Self {
            kind,
            subject,
            user: None,
        }
    }

    fn subject_is_recordable(kind: ResourceKind, subject: &str) -> bool {
        !ACCOUNT_SCOPED.contains(&kind) || !subject.contains(USER_MARK)
    }

    pub fn with_user(
        kind: ResourceKind,
        subject: impl Into<String>,
        user: Option<&str>,
    ) -> crate::core::Result<Self> {
        if let Some(name) = user {
            if !ACCOUNT_SCOPED.contains(&kind) {
                return Err(crate::core::Error::Validation(format!(
                    "a `{}:` row is not keyed by account, so it cannot be for `{}`",
                    kind, name
                )));
            }
            if !account_name_is_well_formed(name) {
                return Err(crate::core::Error::Validation(format!(
                    "`{}` is not an account name this ledger can record",
                    name
                )));
            }
        }
        let subject = subject.into();
        if !Self::subject_is_recordable(kind, &subject) {
            return Err(crate::core::Error::Validation(format!(
                "this `{}:` resource is named `{}`, and a name containing `{USER_MARK}` cannot be \
                 recorded: the ledger row that names it would read back as a different resource \
                 for a different account. Rename it.",
                kind, subject
            )));
        }
        Ok(Self {
            kind,
            subject,
            user: user.map(str::to_string),
        })
    }

    pub fn account_suffix(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// The ledger key of a file Shall placed, from its destination.
    ///
    /// A second caller asks this question from the other end: a `dotfiles:` tree has the
    /// destination in hand and needs to know whether the ledger already claims it. That question
    /// and [`extra_key`]'s `link:` arm must produce the same string or a teardown searches for a
    /// row nothing wrote, so there is one constructor and that arm calls it.
    pub fn link(destination: &Path) -> Self {
        Self::new(ResourceKind::Link, destination.display().to_string())
    }
}

pub fn render(kind: ResourceKind, subject: &str, user: Option<&str>) -> String {
    match user {
        None => format!("{kind}:{subject}"),
        Some(user) => format!("{kind}:{subject}{USER_MARK}{user}"),
    }
}

impl std::fmt::Display for ExtraKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&render(self.kind, &self.subject, self.user.as_deref()))
    }
}

impl std::str::FromStr for ExtraKey {
    type Err = ();

    /// Split at the **first** colon: the kind is one keyword and everything after it belongs to
    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        let (kind, rest) = s.split_once(':').ok_or(())?;
        let kind: ResourceKind = kind.parse()?;
        if !ACCOUNT_SCOPED.contains(&kind) {
            return Ok(Self::new(kind, rest));
        }
        let (subject, user) = split_account_suffix(rest)?;
        Ok(Self {
            kind,
            subject,
            user,
        })
    }
}

pub fn qualify_removal(
    kind: ResourceKind,
    subject: &str,
    user: Option<&str>,
) -> crate::core::Result<String> {
    let key = ExtraKey::with_user(kind, subject, user)?;
    Ok(match &key.user {
        None => key.subject,
        Some(user) => format!("{}{USER_MARK}{user}", key.subject),
    })
}

pub fn split_removal(
    kind: ResourceKind,
    name: &str,
) -> crate::core::Result<(String, Option<String>)> {
    if !ACCOUNT_SCOPED.contains(&kind) {
        return Ok((name.to_string(), None));
    }
    split_account_suffix(name).map_err(|()| {
        crate::core::Error::Validation(format!(
            "`{kind}:{name}` is recorded in the extras ledger, and its account cannot be read \
             back from it. The row is kept rather than acted on, because acting on it would mean \
             choosing an account to change."
        ))
    })
}

/// The stable identity of an applied extra, `<kind>:<id>`. Parseable back into an undo action
/// (see `Extras::reconcile`), and stable across runs so the same declaration always keys
/// the same ledger entry. Returns `None` for statements that are not applied extras (packages,
/// set-math, `use`) — those are tracked elsewhere or not at all.
pub fn extra_key(stmt: &Statement) -> crate::core::Result<Option<ExtraKey>> {
    Ok(match stmt {
        Statement::Link(name, opts) => {
            let subject = match (opts.one("target"), opts.one("user")) {
                (Some(target), user) => {
                    let path = match user {
                        Some(user) => crate::backends::link::resolve_target_for_user(target, user)?,
                        None => crate::backends::link::resolve_target(target)?,
                    };
                    ExtraKey::link(&path)
                }
                (None, Some(_)) => {
                    return Err(crate::core::Error::Validation(
                        "`link:` with @user= requires @target=".into(),
                    ))
                }
                (None, None) => ExtraKey::new(ResourceKind::Link, name),
            };
            Some(subject)
        }
        // `exec:` is deliberately NOT an extra. Extras are nouns whose teardown undoes what
        // they put in place; a verb has no such inverse, and a script that succeeds makes its
        // own `when` false — so wiring it into this ledger would re-run or "undo" it every
        // time the condition swung. Its lifecycle is `locks/exec.toml`, not here (XIII.3).
        //
        // A dotfiles tree is excluded for the opposite reason: its files ARE keyed here, one
        // row per placed file (U22), but the rows come from `Dotfiles::links` — which walks
        // the tree — because this function has only the declaration and a tree's contents are
        // a fact about the disk. That is precisely why the row it documents did not exist for
        // two weeks: nothing was in a position to write it, and four documents said otherwise.
        // `generate:` is excluded for the same reason as `exec:`: it is a verb that runs a
        // command, not a noun with an inverse. Its output declarations ARE nouns and are keyed
        // here individually once merged, but the generate line itself has no teardown.
        Statement::Exec(..) | Statement::Generate(..) | Statement::Dotfiles(..) => None,
        // **A setting is keyed with its scope**, because the teardown resets what the
        // declaration wrote and `@scope=` is the only thing that says where that was. Without
        // it the key is `setting:org.gnome.x/y` for both a user line and a system one, and the
        // removal reset the store's default scope — so deleting a `setting:x@scope=system` line
        // reset the USER key and left the machine-wide value in place, silently.
        //
        // Unscoped keys stay exactly as they were: `@scope=` is written only to override a
        // store's own default, so a key with no suffix means that default, which is what every
        // existing row in every ledger already means.
        Statement::Dir(name, opts) => {
            let path = match opts.one("user") {
                Some(user) => crate::backends::link::resolve_target_for_user(name, user)?,
                None => crate::backends::link::resolve_target(name)?,
            };
            require_absolute_dir(name, &path)?;
            Some(ExtraKey::new(ResourceKind::Dir, path.display().to_string()))
        }
        Statement::Setting(name, opts) => Some(ExtraKey::with_user(
            ResourceKind::Setting,
            match opts.one("scope") {
                Some(scope) => format!("{}@scope={}", name, scope),
                None => name.clone(),
            },
            opts.one("user"),
        )?),
        Statement::Shim(name, opts) => Some(ExtraKey::with_user(
            ResourceKind::Shim,
            name,
            opts.one("user"),
        )?),
        Statement::Service(name, opts) => Some(ExtraKey::with_user(
            ResourceKind::Service,
            name,
            opts.one("user"),
        )?),
        Statement::Schedule(name, opts) => Some(ExtraKey::with_user(
            ResourceKind::Schedule,
            name,
            opts.one("user"),
        )?),
        // Everything else with a keyword is a noun with an inverse: deleting a `firewall:` line
        // closes the port (N5), deleting a `service:` line disables the service.
        //
        // Built from the kind and the subject rather than from `Statement::key()`, so the key
        // this ledger writes and the key it reads back are one construction. `key()` is
        // documented as the *display* form of a line; that the two agree for these kinds is
        // true and is not a promise anybody made.
        _ => match (stmt.kind(), stmt.subject()) {
            (Some(kind), Some(subject)) => Some(ExtraKey::new(kind, subject)),
            _ => None,
        },
    })
}

pub fn options_carried_by_key(kind: ResourceKind) -> &'static [&'static str] {
    match kind {
        ResourceKind::Link => &["target", "user"],
        ResourceKind::Setting => &["scope", "user"],
        ResourceKind::Shim | ResourceKind::Service | ResourceKind::Schedule => &["user"],
        ResourceKind::Repo
        | ResourceKind::Dir
        | ResourceKind::Firewall
        | ResourceKind::Exec
        | ResourceKind::Generate
        | ResourceKind::Dotfiles => &[],
    }
}

pub const SCHEMA: u32 = 2;

pub fn require_absolute_dir(declared: &str, resolved: &Path) -> crate::core::Result<()> {
    if resolved.is_absolute() {
        return Ok(());
    }
    Err(crate::core::Error::Validation(format!(
        "`dir:{declared}` is not an absolute path. A relative one would be read against the \
         directory this Shall happened to be started in, so `dir:{declared}` would place a \
         different directory depending on where you ran it from. Write an absolute path, or \
         start it with `~` to mean your home directory."
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirOrigin {
    PreExisting,
    ShallCreated,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExtraRecord {
    #[serde(with = "resource_kind")]
    pub kind: ResourceKind,
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DirOrigin>,
}

impl ExtraRecord {
    pub fn new(key: &ExtraKey) -> Self {
        Self {
            kind: key.kind,
            subject: key.subject.clone(),
            user: key.user.clone(),
            created: None,
        }
    }

    pub fn parse(wire: &str) -> crate::core::Result<Self> {
        let key = wire.parse::<ExtraKey>().map_err(|()| {
            crate::core::Error::Validation(format!("`{wire}` is not an extras ledger row"))
        })?;
        Ok(Self::new(&key))
    }

    pub fn with_created(mut self, created: Option<DirOrigin>) -> crate::core::Result<Self> {
        if created.is_some() && self.kind != ResourceKind::Dir {
            return Err(crate::core::Error::Validation(format!(
                "a `{}:` row cannot carry `created`; only a `dir:` records whether Shall made the \
                 directory, because only a `dir:` teardown removes something the user may have \
                 had first.",
                self.kind
            )));
        }
        self.created = created;
        Ok(self)
    }

    pub fn key(&self) -> crate::core::Result<ExtraKey> {
        ExtraKey::with_user(self.kind, &self.subject, self.user.as_deref())
    }

    pub fn wire(&self) -> String {
        render(self.kind, &self.subject, self.user.as_deref())
    }

    pub fn owns_removal(&self) -> bool {
        self.kind != ResourceKind::Dir || self.created == Some(DirOrigin::ShallCreated)
    }
}

mod resource_kind {
    use super::ResourceKind;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(kind: &ResourceKind, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(kind.as_str())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ResourceKind, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(|()| {
            serde::de::Error::custom(format!("`{raw}` is not a resource keyword this build has"))
        })
    }
}

fn normalize(records: Vec<ExtraRecord>) -> Vec<ExtraRecord> {
    let mut records = records;
    records.sort();
    records.dedup();
    records
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtrasLedger {
    schema: u32,
    records: Vec<ExtraRecord>,
}

impl Default for ExtrasLedger {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            records: Vec::new(),
        }
    }
}

impl<'de> Deserialize<'de> for ExtrasLedger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Document {
            #[serde(default)]
            applied: Option<Vec<String>>,
            #[serde(default)]
            schema: Option<u32>,
            #[serde(default)]
            records: Vec<ExtraRecord>,
        }

        let document = Document::deserialize(deserializer)?;
        if document.applied.is_some() {
            return Err(D::Error::custom(LEGACY_APPLIED));
        }
        match document.schema {
            None => Err(D::Error::custom(NO_SCHEMA)),
            Some(SCHEMA) => {
                let records = normalize(document.records);
                if let Some(duplicate) = records
                    .windows(2)
                    .find(|pair| pair[0].wire() == pair[1].wire())
                {
                    return Err(D::Error::custom(format!(
                        "`locks/extras.toml` records `{}` more than once. One resource has one \
                         row, and choosing between two rows for the same path would choose what \
                         Shall is allowed to remove.",
                        duplicate[0].wire()
                    )));
                }
                Ok(Self {
                    schema: SCHEMA,
                    records,
                })
            }
            Some(other) => Err(D::Error::custom(format!(
                "`locks/extras.toml` was written for schema {other}, and this build reads \
                 schema {SCHEMA}. A ledger from a different build of Shall is refused rather \
                 than read under rules it was not written for."
            ))),
        }
    }
}

const LEGACY_APPLIED: &str = "\
`locks/extras.toml` is in the schema-1 format — an `applied = [...]` list of keys, with \
nothing in it saying which of them Shall created. Shall does not migrate it: the teardown \
for a `dir:` may only remove a directory Shall made, and a migrated row would claim a \
provenance nobody ever wrote down. Delete the file and run `shall sync -y` once to start a \
fresh one. Nothing on the machine is removed by deleting it; what you give up is the record \
of what a previous sync placed, so read it first if you need that list.";

const NO_SCHEMA: &str = "\
`locks/extras.toml` has no `schema` field, so this build cannot tell what wrote it or what \
its fields mean. Shall does not guess: delete the file and run `shall sync -y` once to start \
a fresh one.";

impl LockFile for ExtrasLedger {
    const WHAT: &'static str = "the extras ledger";
}

impl ExtrasLedger {
    pub fn path_in(locks_dir: &Path) -> PathBuf {
        locks_dir.join("extras.toml")
    }

    pub fn schema(&self) -> u32 {
        self.schema
    }

    pub fn records(&self) -> &[ExtraRecord] {
        &self.records
    }

    pub fn record_for(&self, key: &ExtraKey) -> Option<&ExtraRecord> {
        let wire = key.to_string();
        self.records.iter().find(|record| record.wire() == wire)
    }

    pub fn is_applied(&self, key: &ExtraKey) -> bool {
        self.record_for(key).is_some()
    }

    pub fn drift<'a>(&'a self, declared: &BTreeSet<ExtraKey>) -> Vec<&'a ExtraRecord> {
        let declared: BTreeSet<String> = declared.iter().map(ToString::to_string).collect();
        self.records
            .iter()
            .filter(|record| !declared.contains(&record.wire()))
            .collect()
    }

    pub fn record(&mut self, records: Vec<ExtraRecord>) {
        self.records = normalize(records);
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

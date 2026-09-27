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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::grammar::Options;

    fn key(wire: &str) -> ExtraKey {
        wire.parse()
            .unwrap_or_else(|()| panic!("`{wire}` is a key"))
    }

    fn keys(wires: &[&str]) -> BTreeSet<ExtraKey> {
        wires.iter().map(|w| key(w)).collect()
    }

    fn records(wires: &[&str]) -> Vec<ExtraRecord> {
        wires.iter().map(|w| ExtraRecord::new(&key(w))).collect()
    }

    fn drifted(ledger: &ExtrasLedger, declared: &[&str]) -> Vec<String> {
        ledger
            .drift(&keys(declared))
            .into_iter()
            .map(|record| record.wire())
            .collect()
    }

    fn shown(stmt: Statement) -> Option<String> {
        extra_key(&stmt).unwrap().map(|k| k.to_string())
    }

    #[test]
    fn keys_are_stable_and_parseable_per_kind() {
        assert_eq!(
            shown(Statement::Shim("rg".into(), Options::default())).as_deref(),
            Some("shim:rg")
        );
        assert_eq!(
            shown(Statement::Service("nginx".into(), Options::default())).as_deref(),
            Some("service:nginx")
        );
        assert_eq!(
            shown(Statement::Repo {
                backend: "apt".into(),
                spec: "ppa:x/y".into()
            })
            .as_deref(),
            Some("repo:apt:ppa:x/y")
        );
    }

    /// **What is written is what is read back.** The ledger is a set of these strings on disk,
    /// so `Display` and `FromStr` being inverses is the whole of its wire format.
    ///
    /// A `repo:` subject carries its own colons, which is why the split is at the FIRST one and
    /// why the subject is not split again here: `repo:apt:ppa:x/y` must hand the undo
    /// `apt:ppa:x/y`, whole.
    #[test]
    fn a_key_round_trips_through_the_string_the_ledger_stores() {
        for (text, kind, subject) in [
            ("service:nginx", ResourceKind::Service, "nginx"),
            ("repo:apt:ppa:x/y", ResourceKind::Repo, "apt:ppa:x/y"),
            ("link:/home/u/.vimrc", ResourceKind::Link, "/home/u/.vimrc"),
            ("firewall:22/tcp", ResourceKind::Firewall, "22/tcp"),
        ] {
            let parsed: ExtraKey = text.parse().unwrap_or_else(|_| panic!("`{text}` parses"));
            assert_eq!(parsed, ExtraKey::new(kind, subject));
            assert_eq!(parsed.to_string(), text);
        }

        // A package key is not an extras key, which is the confusion `Statement::kind`'s own
        // doc comment warns about: re-splitting on `:` would read `apt:jq` as the kind `apt`.
        assert_eq!("apt:jq".parse::<ExtraKey>(), Err(()));
        // And a bare word names no kind at all.
        assert_eq!("nginx".parse::<ExtraKey>(), Err(()));
    }

    /// **Every key this ledger can write names a kind the teardown can dispatch on.**
    ///
    /// The teardown is exhaustive over `ResourceKind` now, so the one way a row can still
    /// arrive un-actionable is for the *key* to open with something that is not a keyword. That
    /// is what a `&str` dispatch could never notice: it matched the arms it knew and shrugged
    /// at the rest, and the shrug reported the undo as done.
    #[test]
    fn every_ledger_key_names_a_kind_the_teardown_can_dispatch_on() {
        let o = Options::default;
        let statements = [
            Statement::Shim("rg".into(), o()),
            Statement::Service("nginx".into(), o()),
            Statement::Setting("dark".into(), o()),
            Statement::Link("src".into(), o()),
            Statement::Schedule("nightly".into(), o()),
            Statement::Firewall("22/tcp".into(), o()),
            Statement::Repo {
                backend: "apt".into(),
                spec: "ppa:x/y".into(),
            },
        ];
        for stmt in &statements {
            let key = extra_key(stmt)
                .unwrap()
                .unwrap_or_else(|| panic!("{stmt:?} produced no key"));
            let parsed: ExtraKey = key
                .to_string()
                .parse()
                .unwrap_or_else(|_| panic!("`{key}` does not read back as a key"));
            assert_eq!(parsed, key, "`{key}` does not survive its own round trip");
            assert_eq!(
                Some(key.kind),
                stmt.kind(),
                "`{key}` names a different kind than the statement it came from"
            );
        }

        // And the three that must NOT be keyed: a verb has no inverse, and a tree's rows are
        // the `link:` keys its files were placed under.
        assert!(extra_key(&Statement::Exec("./x.sh".into(), o()))
            .unwrap()
            .is_none());
        assert!(extra_key(&Statement::Generate("./x.sh".into(), o()))
            .unwrap()
            .is_none());
        assert!(extra_key(&Statement::Dotfiles("tree".into(), o()))
            .unwrap()
            .is_none());
    }

    /// **The scope rides the key, because by teardown time the line that carried it is gone.**
    ///
    /// `setting:x@scope=system` and `setting:x` were the same ledger row, so removing the
    /// system-scoped line reset the USER key and left the machine-wide value in place, reporting
    /// success. The removal reads the scope back off the subject; an unscoped key still means
    /// the store's own default, which is what every row written before this said.
    #[test]
    fn a_scoped_setting_is_a_different_row_from_an_unscoped_one() {
        let mut system = Options::default();
        system.insert("scope", "system");
        let scoped = extra_key(&Statement::Setting(
            "org.gnome.desktop/theme".into(),
            system,
        ))
        .expect("a setting is an extra")
        .expect("a setting is an extra");
        assert_eq!(
            scoped.to_string(),
            "setting:org.gnome.desktop/theme@scope=system"
        );

        let plain = extra_key(&Statement::Setting(
            "org.gnome.desktop/theme".into(),
            Options::default(),
        ))
        .expect("a setting is an extra")
        .expect("a setting is an extra");
        assert_eq!(plain.to_string(), "setting:org.gnome.desktop/theme");

        assert_ne!(
            scoped, plain,
            "one row for both scopes is how a system reset became a user reset"
        );
        // Both still round trip, and the scope stays in the subject rather than becoming a
        // second kind: the teardown dispatches on the kind and reads the rest.
        for key in [&scoped, &plain] {
            assert_eq!(key.kind, ResourceKind::Setting);
            assert_eq!(
                key.to_string().parse::<ExtraKey>().as_ref(),
                Ok(key),
                "`{key}` does not read back"
            );
        }
        // And deleting a system line leaves an unscoped declaration of the same key alone.
        let mut ledger = ExtrasLedger::new();
        ledger.record(records(&[&scoped.to_string()]));
        assert_eq!(
            drifted(&ledger, &[&plain.to_string()]),
            vec![scoped.to_string()]
        );
    }

    #[test]
    fn a_package_line_has_no_extra_key() {
        // Packages are tracked by the registry, not this ledger.
        assert!(extra_key(&Statement::Subtract("vim".into()))
            .unwrap()
            .is_none());
    }

    fn account_scoped(kind: ResourceKind, name: &str, user: &str) -> String {
        let mut opts = Options::default();
        opts.set("user", user);
        let stmt = match kind {
            ResourceKind::Shim => Statement::Shim(name.into(), opts),
            ResourceKind::Service => Statement::Service(name.into(), opts),
            ResourceKind::Setting => Statement::Setting(name.into(), opts),
            ResourceKind::Schedule => Statement::Schedule(name.into(), opts),
            other => panic!("{} is not account-scoped", other),
        };
        extra_key(&stmt).unwrap().unwrap().to_string()
    }

    #[test]
    fn two_users_of_one_resource_are_two_rows() {
        for (kind, name) in [
            (ResourceKind::Shim, "rg"),
            (ResourceKind::Service, "nginx"),
            (ResourceKind::Schedule, "nightly"),
            (ResourceKind::Setting, "org.gnome.desktop/theme"),
        ] {
            let alice = account_scoped(kind, name, "alice");
            let bob = account_scoped(kind, name, "bob");
            assert_ne!(alice, bob, "{kind}: two accounts wrote one row");
            assert!(alice.ends_with("@user=alice"), "{alice}");
            assert!(bob.ends_with("@user=bob"), "{bob}");

            let mut ledger = ExtrasLedger::new();
            ledger.record(records(&[&alice]));
            assert_eq!(
                drifted(&ledger, &[&bob]),
                vec![alice.clone()],
                "{kind}: one account's row drifted on the other's declaration"
            );

            let parsed: ExtraKey = alice.parse().expect("the row reads back");
            assert_eq!(parsed.account_suffix(), Some("alice"), "{kind}");
            assert_eq!(parsed.subject, name, "{kind}: the subject is not the name");
        }
    }

    #[test]
    fn an_unscoped_row_is_unchanged_and_is_not_the_scoped_one() {
        for (kind, name, unscoped) in [
            (ResourceKind::Shim, "rg", "shim:rg"),
            (ResourceKind::Service, "nginx", "service:nginx"),
            (ResourceKind::Schedule, "nightly", "schedule:nightly"),
        ] {
            let key = extra_key(&match kind {
                ResourceKind::Shim => Statement::Shim(name.into(), Options::default()),
                ResourceKind::Service => Statement::Service(name.into(), Options::default()),
                _ => Statement::Schedule(name.into(), Options::default()),
            })
            .unwrap()
            .unwrap();
            assert_eq!(key.to_string(), unscoped);
            assert_eq!(key.account_suffix(), None);
            assert_ne!(key, account_scoped(kind, name, "alice").parse().unwrap());
        }
    }

    #[test]
    fn a_setting_records_its_scope_and_its_account_separately() {
        let mut system = Options::default();
        system.set("scope", "system");
        system.set("user", "alice");
        let key = extra_key(&Statement::Setting("org.gnome.x/theme".into(), system))
            .unwrap()
            .unwrap();
        assert_eq!(
            key.to_string(),
            "setting:org.gnome.x/theme@scope=system@user=alice"
        );
        let read: ExtraKey = key.to_string().parse().expect("both suffixes read back");
        assert_eq!(read.account_suffix(), Some("alice"));
        assert_eq!(read.subject, "org.gnome.x/theme@scope=system");
        assert_eq!(read, key);

        let rows: BTreeSet<String> = [
            Options::default(),
            {
                let mut o = Options::default();
                o.set("scope", "system");
                o
            },
            {
                let mut o = Options::default();
                o.set("user", "alice");
                o
            },
            {
                let mut o = Options::default();
                o.set("scope", "system");
                o.set("user", "alice");
                o
            },
        ]
        .into_iter()
        .map(|o| {
            extra_key(&Statement::Setting("k".into(), o))
                .unwrap()
                .unwrap()
                .to_string()
        })
        .collect();
        assert_eq!(rows.len(), 4, "{rows:?}");
    }

    #[test]
    fn the_account_codec_round_trips_adversarial_subjects() {
        let subjects = [
            "rg",
            "a@b",
            "a@@b",
            "@",
            "trailing@",
            "@leading",
            "org.gnome.desktop.theme/night-mode",
            "k@scope=system",
            "apt:ppa:x/y",
            "22/tcp",
            "with space",
            "org.gnome.x/theme@user",
            "unicode-Ω-日本語",
            "quote\"and'apostrophe",
        ];
        for subject in subjects {
            for user in [None, Some("alice"), Some("bob.smith-2"), Some("a")] {
                let key = ExtraKey::with_user(ResourceKind::Shim, subject, user).unwrap();
                let wire = key.to_string();
                let read: ExtraKey = wire
                    .parse()
                    .unwrap_or_else(|_| panic!("`{wire}` does not read back"));
                assert_eq!(read, key, "`{wire}` did not survive its own round trip");
                assert_eq!(read.to_string(), wire);
                assert_eq!(read.account_suffix(), user, "`{wire}` moved the account");
                assert_eq!(read.subject, subject, "`{wire}` did not keep the subject");
            }
        }
    }

    #[test]
    fn a_subject_containing_the_account_marker_is_refused_rather_than_escaped() {
        for subject in ["rg@user=bob", "@user=bob", "x@user=y@user=z"] {
            let err = ExtraKey::with_user(ResourceKind::Shim, subject, Some("alice"))
                .expect_err("a subject that cannot be recorded must be refused");
            assert!(err.to_string().contains("@user="), "{err}");
            assert!(err.to_string().contains("Rename it"), "{err}");
        }
    }

    #[test]
    fn a_malformed_account_suffix_is_refused_at_the_parse() {
        for bad in [
            "shim:rg@user=",
            "shim:@user=alice",
            "shim:rg@user=al ice",
            "shim:rg@user=a:b",
            "shim:rg@user=a@b",
            "shim:rg@user=a\nb",
            "service:nginx@user=",
        ] {
            assert_eq!(
                bad.parse::<ExtraKey>(),
                Err(()),
                "`{bad}` was read as an identity it does not have"
            );
        }
    }

    #[test]
    fn a_kind_with_no_account_domain_never_parses_a_suffix() {
        let row = "link:/home/alice/weird@user=bob";
        let key: ExtraKey = row.parse().expect("a path is a path");
        assert_eq!(key.account_suffix(), None);
        assert_eq!(key.subject, "/home/alice/weird@user=bob");
        assert_eq!(key.to_string(), row, "a path round-trips byte for byte");
    }

    #[test]
    fn a_link_row_is_never_account_scoped() {
        let key = ExtraKey::link(Path::new("/home/alice/.vimrc"));
        assert_eq!(key.to_string(), "link:/home/alice/.vimrc");
        assert_eq!(key.account_suffix(), None);
        let refused = ExtraKey::with_user(ResourceKind::Link, "/home/alice/.vimrc", Some("alice"))
            .expect_err("a link row is a path, not an account");
        assert!(
            refused.to_string().contains("not keyed by account"),
            "{refused}"
        );
    }

    #[test]
    fn a_kind_with_no_account_domain_cannot_be_given_one() {
        for kind in [
            ResourceKind::Repo,
            ResourceKind::Firewall,
            ResourceKind::Link,
            ResourceKind::Dir,
            ResourceKind::Exec,
        ] {
            assert!(
                ExtraKey::with_user(kind, "x", Some("alice")).is_err(),
                "{kind} accepted an account"
            );
        }
    }

    #[test]
    fn per_user_link_and_dir_keys_use_the_resolved_destination() {
        let account = crate::core::account::Account::current().unwrap();
        let mut link_options = Options::default();
        link_options.set("target", "~/.config/shall-link");
        link_options.set("user", account.name());
        let link = extra_key(&Statement::Link("./dotfiles/link".into(), link_options))
            .unwrap()
            .unwrap();
        assert_eq!(
            link.to_string(),
            format!(
                "link:{}",
                account.home().join(".config/shall-link").display()
            )
        );

        let mut dir_options = Options::default();
        dir_options.set("user", account.name());
        let dir = extra_key(&Statement::Dir("~/state".into(), dir_options))
            .unwrap()
            .unwrap();
        assert_eq!(
            dir.to_string(),
            format!("dir:{}", account.home().join("state").display())
        );
    }
    #[test]
    fn drift_is_recorded_minus_declared() {
        let mut ledger = ExtrasLedger::new();
        ledger.record(records(&["service:nginx", "shim:rg", "repo:apt:ppa:x/y"]));
        // The user deleted the service line; the other two remain.
        assert_eq!(
            drifted(&ledger, &["shim:rg", "repo:apt:ppa:x/y"]),
            vec!["service:nginx".to_string()]
        );
    }

    #[test]
    fn nothing_drifts_when_everything_is_still_declared() {
        let mut ledger = ExtrasLedger::new();
        ledger.record(records(&["shim:rg"]));
        assert!(drifted(&ledger, &["shim:rg"]).is_empty());
    }

    #[test]
    fn a_newly_declared_extra_is_not_drift() {
        // A key declared now but not in the ledger is an ADD, not a removal — apply handles
        // it; drift() must not report it.
        let ledger = ExtrasLedger::new();
        assert!(drifted(&ledger, &["shim:new"]).is_empty());
    }

    #[test]
    fn the_ledger_round_trips_through_toml() {
        let mut ledger = ExtrasLedger::new();
        ledger.record(
            records(&["service:nginx", "shim:rg"])
                .into_iter()
                .chain([record("dir:/var/lib/shall", DirOrigin::ShallCreated)])
                .collect(),
        );
        let body = toml::to_string_pretty(&ledger).unwrap();
        let parsed: ExtrasLedger = toml::from_str(&body).unwrap();
        assert_eq!(ledger, parsed);
        assert_eq!(parsed.schema(), SCHEMA);
        assert!(
            body.contains("created = \"shall-created\""),
            "the origin is not on the wire, so a teardown could not read it back:\n{body}"
        );
    }

    fn record(wire: &str, created: DirOrigin) -> ExtraRecord {
        ExtraRecord::new(&key(wire))
            .with_created(Some(created))
            .expect("a `dir:` row carries an origin")
    }

    #[test]
    fn the_schema_one_applied_list_is_refused_rather_than_migrated() {
        let body = "applied = [\"shim:rg\", \"dir:/home/u/state\"]\n";
        let err = toml::from_str::<ExtrasLedger>(body).expect_err("the old format is refused");
        let err = err.to_string();
        assert!(err.contains("schema-1"), "{err}");
        assert!(err.contains("applied"), "{err}");
        assert!(err.contains("shall sync -y"), "{err}");
    }

    #[test]
    fn an_empty_schema_one_list_is_refused_too() {
        assert!(toml::from_str::<ExtrasLedger>("applied = []\n").is_err());
    }

    #[test]
    fn a_missing_file_loads_empty_and_a_schemaless_one_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(ExtrasLedger::load(&tmp.path().join("absent.toml"))
            .unwrap()
            .is_empty());

        let path = tmp.path().join("extras.toml");
        std::fs::write(&path, "records = []\n").unwrap();
        let err = ExtrasLedger::load(&path).expect_err("a file with no schema is refused");
        let err = err.to_string();
        assert!(err.contains("no `schema` field"), "{err}");
        assert!(err.contains("shall sync -y"), "{err}");
    }

    #[test]
    fn a_ledger_written_for_another_schema_is_refused() {
        let err = toml::from_str::<ExtrasLedger>("schema = 3\n").expect_err("refused");
        assert!(err.to_string().contains("schema 3"), "{err}");
    }

    #[test]
    fn only_a_dir_row_may_carry_a_creation_origin() {
        for kind in [
            ResourceKind::Repo,
            ResourceKind::Shim,
            ResourceKind::Schedule,
            ResourceKind::Service,
            ResourceKind::Link,
            ResourceKind::Setting,
            ResourceKind::Firewall,
        ] {
            let record = ExtraRecord::new(&ExtraKey::new(kind, "x"));
            let err = record
                .clone()
                .with_created(Some(DirOrigin::ShallCreated))
                .expect_err("only a `dir:` is ever created by a path Shall made");
            assert!(err.to_string().contains("`dir:`"), "{kind}: {err}");
            assert!(
                record.with_created(None).is_ok(),
                "{kind}: a row that claims nothing is every kind's own"
            );
        }
    }

    #[test]
    fn only_a_shall_created_dir_is_authorized_for_removal() {
        let made = record("dir:/var/lib/shall", DirOrigin::ShallCreated);
        let found = record("dir:/home/u/state", DirOrigin::PreExisting);
        let unknown = ExtraRecord::new(&key("dir:/srv/spool"));
        assert!(made.owns_removal());
        assert!(!found.owns_removal());
        assert!(
            !unknown.owns_removal(),
            "a row with no origin says nothing, and a deletion needs a claim"
        );
        for wire in [
            "shim:rg",
            "service:nginx",
            "link:/home/u/.vimrc",
            "setting:org.gnome.x/theme@user=alice",
        ] {
            assert!(
                ExtraRecord::new(&key(wire)).owns_removal(),
                "{wire}: a teardown that removes a declaration is not gated on provenance"
            );
        }
    }

    #[test]
    fn a_dir_row_round_trips_through_its_fields() {
        let account = crate::core::account::Account::current().unwrap();
        let alice = key("shim:rg@user=alice");
        let bob = key("shim:rg@user=bob");
        let record = ExtraRecord::new(&alice);
        assert_eq!(record.wire(), "shim:rg@user=alice");
        assert_eq!(record.key().unwrap(), alice);
        assert_ne!(record.wire(), ExtraRecord::new(&bob).wire());

        let mine = record2(&format!("dir:{}", account.home().join("state").display()));
        assert_eq!(
            mine.user, None,
            "a `dir:` row is keyed by the path it resolved to"
        );
        assert_eq!(mine.created, Some(DirOrigin::PreExisting));
    }

    fn record2(wire: &str) -> ExtraRecord {
        let parsed = key(wire);
        let created = if parsed.kind == ResourceKind::Dir {
            DirOrigin::PreExisting
        } else {
            return ExtraRecord::new(&parsed);
        };
        ExtraRecord::new(&parsed)
            .with_created(Some(created))
            .expect("a `dir:` row carries an origin")
    }

    #[test]
    fn a_row_too_broken_to_parse_is_still_rendered() {
        let record = ExtraRecord {
            kind: ResourceKind::Shim,
            subject: "rg@user=".into(),
            user: None,
            created: None,
        };
        assert!(record.key().is_err());
        assert_eq!(record.wire(), "shim:rg@user=");
    }

    #[test]
    fn a_kind_this_build_has_no_keyword_for_is_refused_at_the_read() {
        let err = toml::from_str::<ExtrasLedger>(
            "schema = 2\n\n[[records]]\nkind = \"container\"\nsubject = \"podman\"\n",
        )
        .expect_err("an unknown keyword is refused");
        assert!(err.to_string().contains("container"), "{err}");
    }

    #[test]
    fn two_rows_for_one_identity_are_refused() {
        let err = toml::from_str::<ExtrasLedger>(
            "schema = 2\n\n[[records]]\nkind = \"dir\"\nsubject = \"/srv/state\"\ncreated = \"pre-existing\"\n\n[[records]]\nkind = \"dir\"\nsubject = \"/srv/state\"\ncreated = \"shall-created\"\n",
        )
        .expect_err("one path may not carry two removal claims");
        assert!(err.to_string().contains("dir:/srv/state"), "{err}");
        assert!(err.to_string().contains("more than once"), "{err}");
    }
}

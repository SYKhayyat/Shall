// src/app/sync/saved_plan.rs
//
// A saved, reviewable plan artifact — Terraform's `plan -out` / `apply plan` for packages.
// `shall plan --out p.json` freezes exactly what a sync would do; `shall apply p.json`
// executes that captured set. A content hash lets `apply` detect that the world changed
// since the plan was captured.

use crate::app::sync::planner::SyncChanges;
use crate::core::{GraphAction, PackageSpec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Bump when the on-disk plan format changes incompatibly.
pub const PLAN_SCHEMA: u32 = 2;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlanRemoval {
    pub backend: String,
    pub name: String,
}

/// The resource half of a frozen plan (N-2): `link:`, `service:`, `setting:`, `shim:`,
/// `schedule:` and `repo:` lines.
///
/// Keys in the extras ledger's own vocabulary (`link:<resolved destination>`,
/// `service:<name>`), because those are what the teardown acts on and what the guard counts —
/// a plan naming resources some other way would be a third vocabulary for one set of things.
///
/// `plan --help` promises that "the exact plan you inspect is the one you later `apply`". It
/// froze `{"installs": [], "removals": []}` over three unapplied `link:` lines and printed
/// `system already matches desired state`, while `--dry-run sync` on the same tree named all
/// three — and the guard's refusal text sends a user here to see what would be undone.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanResources {
    /// Declared and not in effect.
    #[serde(default)]
    pub place: Vec<String>,
    /// Applied before and declared nowhere now.
    #[serde(default)]
    pub undo: Vec<String>,
    #[serde(default)]
    pub place_intents: Vec<crate::app::apply::ResourceIntent>,
}

impl PlanResources {
    pub fn is_empty(&self) -> bool {
        self.place.is_empty() && self.undo.is_empty()
    }

    pub fn len(&self) -> usize {
        self.place.len() + self.undo.len()
    }

    pub fn intent_attributes(&self, key: &str) -> &[String] {
        self.place_intents
            .iter()
            .find(|intent| intent.key == key)
            .map(|intent| intent.attributes.as_slice())
            .unwrap_or(&[])
    }
}

/// A frozen, serializable sync plan.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedPlan {
    pub schema: u32,
    /// Unix seconds the plan was captured (passed in — no clock in the pure builder).
    pub created_at: Option<i64>,
    /// Stable content hash of the operations, for drift detection at apply time.
    pub desired_hash: String,
    pub installs: Vec<PackageSpec>,
    pub removals: Vec<PlanRemoval>,
    /// The resources this plan would place and undo. `serde(default)` so a plan written before
    /// the field existed still reads as one with no resource work.
    #[serde(default)]
    pub resources: PlanResources,
    /// The variables this plan resolved against (Part IX, IX.6). `apply` resolves the model
    /// against these rather than running the provider again: a provider may read the clock or
    /// shell out, so a fresh resolution at apply time could disagree with the preview and make
    /// every plan with a moving variable spuriously fail its drift check. Auxiliary to the hash
    /// — the executed operations are `installs`/`removals`, which the hash protects.
    #[serde(default)]
    pub vars: std::collections::BTreeMap<String, crate::model::vars::Value>,
}

impl SavedPlan {
    /// Freeze computed sync changes into a saved plan.
    pub fn from_changes(
        changes: &SyncChanges,
        resources: &crate::app::apply::ResourceChanges,
        created_at: Option<i64>,
    ) -> Self {
        let mut installs = Vec::new();
        let mut removals = Vec::new();
        for w in changes.graph.node_weights() {
            match w {
                GraphAction::Install(spec) => installs.push(spec.clone()),
                GraphAction::Remove { name, backend } => removals.push(PlanRemoval {
                    backend: backend.clone(),
                    name: name.clone(),
                }),
            }
        }
        let resources = PlanResources {
            place: resources.place.clone(),
            undo: resources.undo.clone(),
            place_intents: resources.place_intents.clone(),
        };
        let desired_hash = hash_plan(&installs, &removals, &resources);
        Self {
            schema: PLAN_SCHEMA,
            created_at,
            desired_hash,
            installs,
            removals,
            resources,
            vars: std::collections::BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.installs.is_empty() && self.removals.is_empty() && self.resources.is_empty()
    }

    /// Recompute the hash from the plan's own contents (to detect a hand-edited file).
    pub fn recomputed_hash(&self) -> String {
        hash_plan(&self.installs, &self.removals, &self.resources)
    }
}

/// Stable content hash over the plan's operations. Order-independent; ignores internal
/// `__`-prefixed provenance options so they don't perturb equality. Pure — unit tested.
pub fn hash_plan(
    installs: &[PackageSpec],
    removals: &[PlanRemoval],
    resources: &PlanResources,
) -> String {
    let mut keys: Vec<String> = Vec::new();
    for s in installs {
        let mut opts: Vec<String> = s
            .options
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| format!("{}={}", k, v.join(",")))
            .collect();
        opts.sort();
        keys.push(format!("I:{}:{}|{}", s.backend, s.name, opts.join(",")));
    }
    for r in removals {
        keys.push(format!("R:{}:{}", r.backend, r.name));
    }
    // In the hash, so a hand-edited resource list is caught by the same integrity check the
    // package lists get. Left out, `apply` would happily place resources nobody reviewed.
    for key in &resources.place {
        keys.push(format!("P:{}", key));
    }
    for key in &resources.undo {
        keys.push(format!("U:{}", key));
    }
    for intent in &resources.place_intents {
        keys.push(format!("D:{}:{}", intent.key, intent.attributes.join(",")));
    }
    keys.sort();

    let mut h = Sha256::new();
    for k in &keys {
        h.update(k.as_bytes());
        h.update(b"\n");
    }
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(backend: &str, name: &str, opt: Option<(&str, &str)>) -> PackageSpec {
        let mut options = crate::config::grammar::Options::default();
        if let Some((k, v)) = opt {
            options.insert(k.to_string(), v.to_string());
        }
        PackageSpec {
            name: name.into(),
            backend: backend.into(),
            options,
            requires: vec![],
            present: true,
        }
    }

    #[test]
    fn hash_is_order_independent() {
        let a = vec![spec("apt", "htop", None), spec("cargo", "ripgrep", None)];
        let b = vec![spec("cargo", "ripgrep", None), spec("apt", "htop", None)];
        assert_eq!(
            hash_plan(&a, &[], &PlanResources::default()),
            hash_plan(&b, &[], &PlanResources::default())
        );
    }

    #[test]
    fn hash_ignores_internal_provenance_options() {
        let a = vec![spec("apt", "htop", Some(("__source", "local.txt")))];
        let b = vec![spec("apt", "htop", Some(("__source", "module:dev")))];
        assert_eq!(
            hash_plan(&a, &[], &PlanResources::default()),
            hash_plan(&b, &[], &PlanResources::default())
        );
    }

    #[test]
    fn a_plan_carries_its_variables_across_a_round_trip() {
        use crate::model::vars::Value;
        let mut plan = SavedPlan::from_changes(
            &crate::app::sync::planner::SyncChanges::default(),
            &crate::app::apply::ResourceChanges::default(),
            Some(1),
        );
        plan.vars
            .insert("role".to_string(), Value::Str("travel".into()));
        plan.vars.insert("cores".to_string(), Value::Num(8.0));
        let json = serde_json::to_string(&plan).unwrap();
        let back: SavedPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(back.vars["role"], Value::Str("travel".into()));
        assert_eq!(back.vars["cores"], Value::Num(8.0));
    }

    #[test]
    fn a_plan_with_no_vars_field_deserializes_to_empty() {
        // The field is `serde(default)`, so a plan written before it existed still reads.
        let raw =
            r#"{"schema":2,"created_at":null,"desired_hash":"x","installs":[],"removals":[]}"#;
        let plan: SavedPlan = serde_json::from_str(raw).unwrap();
        assert!(plan.vars.is_empty());
    }

    #[test]
    fn hash_changes_with_real_options_and_removals() {
        let base = vec![spec("apt", "htop", None)];
        let versioned = vec![spec("apt", "htop", Some(("version", "3.0")))];
        assert_ne!(
            hash_plan(&base, &[], &PlanResources::default()),
            hash_plan(&versioned, &[], &PlanResources::default())
        );

        let with_removal = hash_plan(
            &base,
            &[PlanRemoval {
                backend: "apt".into(),
                name: "vim".into(),
            }],
            &PlanResources::default(),
        );
        assert_ne!(
            hash_plan(&base, &[], &PlanResources::default()),
            with_removal
        );
    }

    fn resources_for(statement: &crate::config::grammar::Statement) -> PlanResources {
        use crate::app::apply::desired_state_attributes;
        let key = crate::core::extras_lock::extra_key(statement)
            .expect("the declaration keys")
            .expect("it is a resource");
        let attributes = desired_state_attributes(statement, key.kind);
        PlanResources {
            place: vec![key.to_string()],
            place_intents: vec![crate::app::apply::ResourceIntent {
                key: key.to_string(),
                attributes,
            }],
            ..Default::default()
        }
    }

    fn dir(path: &str, options: &[(&str, &str)]) -> crate::config::grammar::Statement {
        let mut opts = crate::config::grammar::Options::default();
        for (name, value) in options {
            opts.set(*name, *value);
        }
        crate::config::grammar::Statement::Dir(path.to_string(), opts)
    }

    fn link(path: &str, options: &[(&str, &str)]) -> crate::config::grammar::Statement {
        let mut opts = crate::config::grammar::Options::default();
        for (name, value) in options {
            opts.set(*name, *value);
        }
        opts.set("target", path);
        crate::config::grammar::Statement::Link("source".to_string(), opts)
    }

    fn artifact(resources: &PlanResources) -> String {
        serde_json::to_string(&SavedPlan {
            schema: PLAN_SCHEMA,
            created_at: None,
            desired_hash: hash_plan(&[], &[], resources),
            installs: vec![],
            removals: vec![],
            resources: resources.clone(),
            vars: Default::default(),
        })
        .expect("a plan serialises")
    }

    #[test]
    fn a_directories_mode_is_part_of_the_plan_and_not_only_its_key() {
        let path = "/srv/state";
        let tight = resources_for(&dir(path, &[("mode", "0700")]));
        let loose = resources_for(&dir(path, &[("mode", "0750")]));

        assert_eq!(
            tight.place, loose.place,
            "the mode is not the key's business, and the key must not widen to hold it: the \
             ledger addresses a directory by where it is"
        );
        assert_ne!(
            tight.intent_attributes(tight.place[0].as_str()),
            loose.intent_attributes(loose.place[0].as_str()),
            "0700 and 0750 froze the same attributes"
        );
        assert_ne!(
            hash_plan(&[], &[], &tight),
            hash_plan(&[], &[], &loose),
            "a plan that cannot tell 0700 from 0750 passes its own drift check on a changed mode"
        );
        assert_ne!(artifact(&tight), artifact(&loose));
    }

    #[test]
    fn a_directories_owner_is_part_of_the_plan_and_not_only_its_key() {
        let path = "/srv/state";
        let alice = resources_for(&dir(path, &[("owner", "alice")]));
        let bob = resources_for(&dir(path, &[("owner", "bob")]));

        assert_eq!(alice.place, bob.place, "the owner widened the key");
        assert_ne!(artifact(&alice), artifact(&bob));
    }

    #[test]
    fn a_links_owner_is_part_of_the_plan_and_not_only_its_key() {
        let path = "/etc/ssh/ssh_config";
        let alice = resources_for(&link(path, &[("owner", "alice")]));
        let bob = resources_for(&link(path, &[("owner", "bob")]));

        assert_eq!(alice.place, bob.place, "the owner widened the key");
        assert_ne!(artifact(&alice), artifact(&bob));
    }

    #[test]
    fn an_unchanged_declaration_freezes_a_byte_identical_plan() {
        let path = "/srv/state";
        let first = artifact(&resources_for(&dir(
            path,
            &[("mode", "0700"), ("owner", "alice")],
        )));
        let second = artifact(&resources_for(&dir(
            path,
            &[("mode", "0700"), ("owner", "alice")],
        )));
        assert_eq!(first, second, "an untouched tree froze two different plans");
    }

    #[test]
    fn the_options_a_key_already_carries_are_not_repeated_in_the_fingerprint() {
        let account = crate::core::account::Account::current()
            .expect("the invoking account has a passwd entry")
            .name()
            .to_string();
        for (statement, key_fragments, uncarried) in [
            (
                {
                    let mut opts = crate::config::grammar::Options::default();
                    opts.set("value", "prefer-dark");
                    opts.set("scope", "system");
                    opts.set("user", &account);
                    opts.set("owner", "the-owner");
                    crate::config::grammar::Statement::Setting("org.gnome.x/theme".into(), opts)
                },
                vec!["@scope=system".to_string(), format!("@user={account}")],
                vec!["owner=the-owner"],
            ),
            (
                {
                    let mut opts = crate::config::grammar::Options::default();
                    opts.set("target", "/etc/motd");
                    opts.set("owner", "the-owner");
                    opts.set("user", &account);
                    crate::config::grammar::Statement::Link("source".into(), opts)
                },
                vec!["/etc/motd".to_string()],
                vec!["owner=the-owner"],
            ),
        ] {
            let resources = resources_for(&statement);
            let key = &resources.place[0];
            let attributes = resources.intent_attributes(key.as_str());
            for fragment in &key_fragments {
                assert!(key.contains(fragment), "{fragment} is not in the key {key}");
            }
            for carried in ["scope", "user", "target"] {
                assert!(
                    !attributes.iter().any(|a| a.starts_with(&format!("{carried}="))),
                    "{carried}= is in the key already, and the fingerprint repeated it: {attributes:?}"
                );
            }
            for expected in &uncarried {
                assert!(
                    attributes.iter().any(|a| a == expected),
                    "the option the key does not carry is missing: {expected:?} not in {attributes:?}"
                );
            }
        }
    }

    #[test]
    fn a_payload_option_is_recorded_as_a_digest_and_never_as_its_text() {
        let resources = resources_for(&crate::config::grammar::Statement::Link(
            "source".to_string(),
            {
                let mut opts = crate::config::grammar::Options::default();
                opts.set("target", "/etc/motd");
                opts.set("content", "the secret body");
                opts
            },
        ));
        let text = artifact(&resources);
        assert!(
            !text.contains("the secret body"),
            "a plan file a user leaves on disk carried the content it would write: {text}"
        );
        assert!(text.contains("content=#"), "{text}");
    }

    #[test]
    fn the_plan_schema_is_bumped_so_an_artifact_without_attributes_is_refused() {
        assert_eq!(
            PLAN_SCHEMA, 2,
            "a plan that cannot say what it would write is not this format; the number is the \
             refusal, and it moved when the attributes arrived"
        );
    }
}

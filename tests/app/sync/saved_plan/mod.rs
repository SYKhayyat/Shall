pub use serde::{Deserialize, Serialize};
pub use sha2::{Digest, Sha256};
use shall::app::apply::desired_state_attributes;
pub use shall::app::sync::planner::SyncChanges;
pub use shall::app::sync::saved_plan::*;
pub use shall::core::{GraphAction, PackageSpec};

fn spec(backend: &str, name: &str, opt: Option<(&str, &str)>) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
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
fn resources_for(statement: &shall::config::grammar::Statement) -> PlanResources {
    let key = shall::core::extras_lock::extra_key(statement)
        .expect("the declaration keys")
        .expect("it is a resource");
    let attributes = desired_state_attributes(statement, key.kind);
    PlanResources {
        place: vec![key.to_string()],
        place_intents: vec![shall::app::apply::ResourceIntent {
            key: key.to_string(),
            attributes,
        }],
        ..Default::default()
    }
}
fn an_absolute_dir() -> &'static str {
    if cfg!(windows) {
        r"C:\srv\state"
    } else {
        "/srv/state"
    }
}
fn dir(path: &str, options: &[(&str, &str)]) -> shall::config::grammar::Statement {
    let mut opts = shall::config::grammar::Options::default();
    for (name, value) in options {
        opts.set(*name, *value);
    }
    shall::config::grammar::Statement::Dir(path.to_string(), opts)
}
fn link(path: &str, options: &[(&str, &str)]) -> shall::config::grammar::Statement {
    let mut opts = shall::config::grammar::Options::default();
    for (name, value) in options {
        opts.set(*name, *value);
    }
    opts.set("target", path);
    shall::config::grammar::Statement::Link("source".to_string(), opts)
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
mod a_directories_mode_is_part_of_the_plan_and_not_only_its_key;
mod a_directories_owner_is_part_of_the_plan_and_not_only_its_key;
mod a_links_owner_is_part_of_the_plan_and_not_only_its_key;
mod a_payload_option_is_recorded_as_a_digest_and_never_as_its_text;
mod a_plan_carries_its_variables_across_a_round_trip;
mod a_plan_with_no_vars_field_deserializes_to_empty;
mod an_unchanged_declaration_freezes_a_byte_identical_plan;
mod hash_changes_with_real_options_and_removals;
mod hash_ignores_internal_provenance_options;
mod hash_is_order_independent;
mod the_options_a_key_already_carries_are_not_repeated_in_the_fingerprint;
mod the_plan_schema_is_bumped_so_an_artifact_without_bare_provenance_is_refused;

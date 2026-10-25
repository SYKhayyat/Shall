pub use serde::{Deserialize, Serialize};
pub use shall::config::grammar::{Options, ResourceKind, Statement};
pub use shall::core::extras_lock::*;
pub use shall::core::ledger::LockFile;
use std::collections::BTreeSet;
pub use std::path::{Path, PathBuf};

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
fn record(wire: &str, created: DirOrigin) -> ExtraRecord {
    ExtraRecord::new(&key(wire))
        .with_created(Some(created))
        .expect("a `dir:` row carries an origin")
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
mod a_dir_row_round_trips_through_its_fields;
mod a_key_round_trips_through_the_string_the_ledger_stores;
mod a_kind_this_build_has_no_keyword_for_is_refused_at_the_read;
mod a_kind_with_no_account_domain_cannot_be_given_one;
mod a_kind_with_no_account_domain_never_parses_a_suffix;
mod a_ledger_written_for_another_schema_is_refused;
mod a_link_row_is_never_account_scoped;
mod a_malformed_account_suffix_is_refused_at_the_parse;
mod a_missing_file_loads_empty_and_a_schemaless_one_is_refused;
mod a_newly_declared_extra_is_not_drift;
mod a_package_line_has_no_extra_key;
mod a_row_too_broken_to_parse_is_still_rendered;
mod a_scoped_setting_is_a_different_row_from_an_unscoped_one;
mod a_setting_records_its_scope_and_its_account_separately;
mod a_subject_containing_the_account_marker_is_refused_rather_than_escaped;
mod an_empty_schema_one_list_is_refused_too;
mod an_unscoped_row_is_unchanged_and_is_not_the_scoped_one;
mod drift_is_recorded_minus_declared;
mod every_ledger_key_names_a_kind_the_teardown_can_dispatch_on;
mod keys_are_stable_and_parseable_per_kind;
mod nothing_drifts_when_everything_is_still_declared;
mod only_a_dir_row_may_carry_a_creation_origin;
mod only_a_shall_created_dir_is_authorized_for_removal;
mod per_user_link_and_dir_keys_use_the_resolved_destination;
mod the_account_codec_round_trips_adversarial_subjects;
mod the_ledger_round_trips_through_toml;
mod the_schema_one_applied_list_is_refused_rather_than_migrated;
mod two_rows_for_one_identity_are_refused;
mod two_users_of_one_resource_are_two_rows;

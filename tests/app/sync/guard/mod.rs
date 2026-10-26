//! The removal guard's own tests, moved out of `src/app/sync/guard.rs`.
//!
//! **The mirror is the rule.** `src/app/sync/guard.rs` owns `tests/app/sync/guard/`, and every
//! test for the guard lives in a file of its own, named after the assertion it makes. The helpers
//! the assertions share stay here, in the module file, because a helper in `src/` could not be
//! reached from a test crate — `tests/` links the library without `cfg(test)`.
//!
//! **What the move cost, and what it bought.** `src/app/sync/guard.rs` was 3,473 lines, of which
//! 1,944 were this module, and it sat against a recorded ceiling of 3,500 in
//! `TOO_BIG_FOR_NOW` — over the line because of its own tests. The code is 1,530 lines, so the
//! ceiling row is retired rather than raised.
//!
//! **Eight items in `src/app/sync/guard.rs` are `pub` for this module's sake**, and the reason is
//! in the register: a `pub fn` reached only from tests is what `a_pub_fn_nobody_calls_is_dead_tests`
//! says a test double looks like, and policing those "would make this a gate people delete". None
//! of the eight mints authority — C3 keeps `Reaped`'s fields private because a token must not be
//! mintable, and these ask the guard rather than standing in for it.

pub use shall::app::sync::guard::*;
pub use shall::backends::BackendRegistry;
pub use shall::config::Config;
pub use std::collections::HashSet;
pub use std::sync::Arc;

// The helpers the assertions below share.

fn pairs(names: &[&str]) -> Vec<(String, String)> {
    names
        .iter()
        .map(|n| ("apt".to_string(), n.to_string()))
        .collect()
}

/// Every per-kind ceiling set to the same number, so a test that says "a limit of two" means
/// it for whichever kind it is about. `Y20` split them and `N8` split ports off again; a
/// helper that moved only one would have left the other kinds' tests measuring against the
/// untouched default of twenty and passing for no reason.
fn config_with(max: usize) -> Config {
    Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec!["python3".into(), "libpam*".into()],
            unprotected_packages: Vec::new(),
            max_removals: max,
            max_extra_removals: max,
            max_port_closures: max,
            ..Default::default()
        },
        ..Config::default()
    }
}

fn desired(
    specs: &[(&str, &str, Option<&str>)],
) -> std::collections::HashMap<String, Vec<shall::core::PackageSpec>> {
    let mut m: std::collections::HashMap<String, Vec<shall::core::PackageSpec>> =
        std::collections::HashMap::new();
    for (backend, name, version) in specs {
        let mut options = shall::config::grammar::Options::default();
        if let Some(v) = version {
            options.set("version", v.to_string());
        }
        m.entry(backend.to_string())
            .or_default()
            .push(shall::core::PackageSpec {
                name: name.to_string(),
                backend: backend.to_string(),
                options,
                requires: vec![],
                present: true,
            });
    }
    m
}

/// Every kind the extras teardown can undo, keyed the way the ledger keys it.
fn extras(keys: &[&str]) -> Vec<(String, String)> {
    extra_removal_pairs(&keys.iter().map(|k| k.to_string()).collect::<Vec<_>>())
}

fn ports(specs: &[&str]) -> Vec<(String, String)> {
    specs
        .iter()
        .map(|s| ("firewall".to_string(), s.to_string()))
        .collect()
}

// ---- The OS-essential protection, driven against a backend that reports one. -----------
//
// **`cargo mutants` over this file on 2026-08-13 found the one hole in it.** 125 mutants:
// 101 killed by the unit tests, 11 rejected by the compiler, and of the 13 survivors the
// integration suite killed all but four. Three of the four are this function:
//
// ```text
// MISSED  replace essential_names -> HashSet<String> with HashSet::new()
// MISSED  replace essential_names -> HashSet<String> with HashSet::from_iter([String::new()])
// MISSED  replace essential_names -> HashSet<String> with HashSet::from_iter(["xyzzy".into()])
// ```
//
// Measured rather than asserted: a Linux binary carrying the first mutation was built and
// run against a clean one on the same image and scenario. The clean binary refused with
// exit 3 and five protections â€” `tar`, `sed`, `grep`, `gzip`, `findutils`. The mutant was
// silent, exited 1, and attempted the removal. **The machine survived only because apt
// refuses to remove its own `Essential: yes` packages** â€” a second line of defence Shall
// does not own, does not check for, and cannot assume of every backend.
//
// **Nothing in this repository caught it.** Not the 1,814 lib tests, not the 535
// integration tests, and not the 425-check container harness â€” the mutant binary swept the
// `tools` image at 424 pass / 1 fail, the same single failure the clean binary has. Even
// `protected includes a system essential`, which the harness greps, passes: that command
// reports the *static* config rules and is byte-identical between the two binaries.
//
// **The reason is a missing input, not a missing assertion.** The suite is hermetic and no
// mock had ever reported an essential set, so `essential_names` returned empty in every test
// that has ever run â€” and a function that always returns empty is indistinguishable from one
// hard-coded to. `Essentials` below is that input.

/// A backend that answers the essential query, which no other test fixture in the tree does.
struct Essentials {
    name: String,
    essential: Vec<String>,
    /// The audit's case: a manager having a bad day. `true` and the query errors instead
    /// of answering â€” the shape a real `apt` produces exactly when its answer matters most.
    fails: bool,
    listings: shall::core::installed::InstalledListings,
}

#[async_trait::async_trait]
impl shall::core::manager::BackendCore for Essentials {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl shall::core::manager::Queryable for Essentials {
    fn installed_cache(&self) -> (&shall::core::installed::InstalledListings, &str) {
        (&self.listings, &self.name)
    }
    async fn fetch_installed(&self) -> shall::core::Result<Vec<shall::core::Package>> {
        Ok(Vec::new())
    }
    async fn list_manual(&self) -> shall::core::Result<Vec<shall::core::Package>> {
        Ok(Vec::new())
    }
    async fn info(&self, _: &str) -> shall::core::Result<Option<shall::core::Package>> {
        Ok(None)
    }
    async fn fetch_essential(&self) -> shall::core::Result<Vec<String>> {
        if self.fails {
            return Err(shall::core::Error::Other(
                "apt-get locked by pid 4242 (fixture)".into(),
            ));
        }
        Ok(self.essential.clone())
    }
}

fn registry_reporting(backend: &str, essential: &[&str]) -> Arc<BackendRegistry> {
    registry_with(&[(backend, essential, false)])
}

/// One registry, several backends, each saying whether its essential query answers.
fn registry_with(rows: &[(&str, &[&str], bool)]) -> Arc<BackendRegistry> {
    let mut reg = BackendRegistry::new();
    for (backend, essential, fails) in rows {
        let fake = Arc::new(Essentials {
            name: (*backend).to_string(),
            essential: essential.iter().map(|s| s.to_string()).collect(),
            fails: *fails,
            listings: Default::default(),
        });
        reg.register(Arc::new(
            shall::core::manager::BackendCapabilities::builder(fake.clone())
                .with_queryable(fake)
                .build(),
        ));
    }
    Arc::new(reg)
}

// One file per assertion.

mod a_backend_with_no_essential_concept_contributes_nothing;
mod a_batch_of_unmodelled_mutations_answers_the_total_ceiling;
mod a_deliberate_purge_ignores_the_count_but_never_protection;
mod a_manager_that_could_not_answer_essentials_stops_the_unwind;
mod a_mass_flag_moves_the_count_onto_the_report_rather_than_deleting_it;
mod a_name_no_line_can_hold_is_never_removed;
mod a_package_the_os_calls_essential_is_never_removed;
mod a_port_closure_answers_to_the_port_ceiling_and_no_other;
mod a_protected_name_stops_a_teardown_of_every_kind;
mod a_protected_package_stops_the_whole_unwind;
mod a_protected_port_is_refused_whatever_the_count;
mod a_removal_offers_a_basename_only_when_it_has_one_to_offer;
mod a_resource_is_refused_by_a_rule_and_not_by_an_accident;
mod a_set_over_two_ceilings_is_told_about_both;
mod a_teardown_refusal_does_not_advise_a_command_that_cannot_take_it;
mod a_vet_decides_without_spending_the_budget;
mod allow_mass_install_clears_the_install_ceiling;
mod allow_mass_removal_answers_a_teardown_count_but_never_its_protection;
mod allow_mass_removal_answers_the_count_and_nothing_else;
mod an_empty_guard_table_objects_to_nothing;
mod an_essential_query_failure_refuses_removals_through_that_backend_only;
mod an_unwind_with_nothing_protected_is_allowed;
mod both_mass_flags_answer_the_total_and_only_the_total;
mod deny_packages_refuses_an_install_case_insensitively;
mod each_removal_kind_spends_only_its_own_budget;
mod essential_names_reports_what_the_backend_says_is_essential;
mod every_objection_describes_itself;
mod every_scope_names_itself_and_no_two_answer_alike;
mod install_at_the_ceiling_is_allowed;
mod install_ceiling_is_off_by_default;
mod install_over_the_ceiling_is_refused;
mod mass_removal_is_refused_even_when_nothing_is_protected;
mod max_removals_zero_disables_the_count_check;
mod no_extra_is_refused_merely_for_not_being_a_package_line;
mod no_setting_can_opt_a_command_out_of_the_guard;
mod packages_and_teardowns_answer_to_their_own_ceilings;
mod pinned_only_requires_a_concrete_version;
mod protected_package_is_refused_even_when_alone;
mod refusal_message_leads_with_the_count_and_caps_the_list;
mod small_ordinary_removal_is_allowed;
mod the_announcement_is_made_only_by_a_run_that_needed_one;
mod the_announcement_names_the_flag_the_run_passed_and_the_ceiling_it_cleared;
mod the_capped_list_claims_more_only_when_there_is_more;
mod the_deliberate_vet_spends_nothing_either;
mod the_preview_sees_the_total_the_enforcer_will;
mod the_refusal_names_the_package_it_refused_to_undo;
mod the_removal_ceiling_fires_past_the_limit_and_not_at_it;
mod the_second_capped_list_claims_more_only_when_there_is_more;
mod the_total_catches_what_every_per_kind_ceiling_passes;
mod the_total_ceiling_fires_past_the_limit_and_not_at_it;
mod the_total_ceiling_refusal_offers_both_flags;
mod the_total_counts_additions_as_well_as_removals;
mod the_total_is_off_by_default_and_when_zero;
mod two_teardown_phases_of_one_command_share_one_budget;
mod undeclaring_a_resource_is_still_allowed;
mod unprotect_wins_over_a_config_rule;
mod unprotect_wins_over_the_os_essential_flag;
mod unprotecting_cannot_release_a_name_that_cannot_be_declared;
mod vet_refuses_what_enforce_refuses_and_says_it_the_same_way;
mod yes_does_not_override_the_guard;
mod yes_does_not_override_the_install_ceiling;

mod a_firewall_teardown_is_a_removal_tests;

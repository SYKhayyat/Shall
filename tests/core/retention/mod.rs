pub use chrono::{DateTime, Duration, Utc};
pub use serde::{Deserialize, Serialize};
pub use shall::core::retention::*;
use std::collections::HashSet;

fn at(days_ago: i64) -> DateTime<Utc> {
    // A fixed reference minus N days, so tests are deterministic.
    DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
        - Duration::days(days_ago)
}
fn now() -> DateTime<Utc> {
    at(0)
}
fn item(id: &str, days_ago: i64) -> RetentionItem {
    RetentionItem::new(id, at(days_ago))
}
mod a_label_is_matched_the_same_way_an_id_is;
mod a_pin_beside_a_rule_still_only_saves;
mod a_pin_list_on_its_own_is_not_permission_to_delete;
mod age_rule_never_wipes_the_last_entry;
mod inactive_policy_deletes_nothing;
mod keep_days_keeps_recent_and_the_floor;
mod keep_last_keeps_newest_n_and_deletes_the_rest;
mod pins_and_labels_always_survive;
mod union_of_count_and_age;

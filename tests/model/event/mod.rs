use serde::Serialize;
use serde_json::json;
pub use shall::model::event::*;
mod an_unknown_event_does_not_parse;
mod every_event_produces_the_same_envelope;
mod every_event_round_trips_through_its_name;
mod stdin_is_newline_terminated;
mod the_payload_is_versioned_and_names_its_event;

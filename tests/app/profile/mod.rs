pub use shall::app::profile::capitalize;
pub use shall::app::profile::*;
pub use shall::app::sync::{ChangePlanner, PlanScope, SyncEngine};
pub use shall::app::vocab::Vocab;
pub use shall::app::Machinery;
pub use shall::config::grammar::Origin;
pub use shall::config::parser::HostFacts;
pub use shall::core::{Error, Result};
pub use shall::model::profiles::{
    blocks_in_active, describe_gate, parse_active, read_active, remove_from_active, ProfileLoader,
};
pub use shall::model::Layout;
pub use tracing::{info, instrument};
mod the_suggestion_capitalizes_the_name_you_typed;

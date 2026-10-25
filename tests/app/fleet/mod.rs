use clap::CommandFactory;
pub use serde_json::{json, Value};
pub use shall::app::fleet::*;
pub use shall::config::Config;
pub use shall::core::{Error, Result};
pub use tracing::{info, warn};

const DRIFTED: &str = r#"[
    {"section":"config","ok":true,"summary":"9 package(s) declared","next":null,
     "counts":{"declared":9,"resources":0}},
    {"section":"drift","ok":false,"summary":"2 to install, 1 to remove, nothing else",
     "next":"shall sync","counts":{"install":2,"remove":1,"skipped":0,"unverifiable":0}},
    {"section":"unmanaged","ok":false,"summary":"3 package(s) `shall adopt` would take",
     "next":"shall adopt","counts":{"unmanaged":3}}
]"#;
const CONVERGED: &str = r#"[
    {"section":"drift","ok":true,"summary":"the machine matches your files","next":null,
     "counts":{"install":0,"remove":0,"skipped":0,"unverifiable":0}},
    {"section":"unmanaged","ok":true,"summary":"everything you chose is managed",
     "next":null,"counts":{"unmanaged":0}}
]"#;
mod a_converged_host_reports_in_sync;
mod a_drifted_host_reports_its_counts;
mod a_host_that_looks_like_an_ssh_option_is_refused;
mod a_missing_count_is_nought;
mod an_answer_that_is_not_a_check_report_is_an_error;
mod in_sync_logic;
mod resource_drift_alone_is_still_drift;
mod what_the_remote_is_asked_to_run;

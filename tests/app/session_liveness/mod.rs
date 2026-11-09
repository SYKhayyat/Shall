pub use shall::app::session_liveness::*;
pub use std::path::{Path, PathBuf};

/// A process table that answers for pids the test names, and nothing else.
///
/// The trait exists for this: the risk is answering "gone" about a session that is running, and a
/// rule exercisable only by killing a real shell is a rule nobody exercises.
pub struct Table {
    pub alive: Vec<u32>,
}

impl Processes for Table {
    fn pid_alive(&self, pid: u32) -> bool {
        self.alive.contains(&pid)
    }
    fn any_named(&self, _name: &str) -> bool {
        false
    }
}

fn stamp(id: &str, pid: u32) -> SessionStamp {
    SessionStamp {
        session_id: id.to_string(),
        pid,
    }
}

mod a_session_whose_process_is_gone_stops_protecting;

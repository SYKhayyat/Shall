//! The one raw-mode/alternate-screen guard both TUIs enter through.
//!
//! **Restoration belongs to `Drop`, not to the happy path.** Both browsers used to
//! `enable_raw_mode()`, run a loop, and restore afterwards — so a panic mid-loop, or any `?`
//! between setup and restore, unwound past the restore and left the user's terminal in raw
//! mode on the alternate screen: no echo, no scrollback, until they typed `reset`. The guard
//! makes the restore unconditional, panic or return.

use ratatui::crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

use crate::core::Result;

pub struct RawScreenGuard {
    /// Held so the terminal cannot be dropped while a draw is in flight.
    _private: (),
}

impl RawScreenGuard {
    pub fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(e.into());
        }
        let _ = stdout.flush();
        Ok(Self { _private: () })
    }
}

impl Drop for RawScreenGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        let _ = stdout.flush();
    }
}

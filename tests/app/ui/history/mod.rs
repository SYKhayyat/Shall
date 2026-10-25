pub use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
};
pub use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
pub use shall::app::ui::history::*;
pub use shall::core::Result;
use std::io;

fn cv(short: &str, subject: &str, changes: &[&str]) -> CommitView {
    CommitView {
        short: short.into(),
        date: "2026-07-15".into(),
        subject: subject.into(),
        full_hash: format!("{}0000000000000000000000000000000000000", short),
        changes: changes.iter().map(|s| s.to_string()).collect(),
        signature: "unsigned".to_string(),
    }
}
mod a_rollback_targets_the_full_hash;
mod commit_row_shows_hash_and_subject;
mod detail_lines_note_a_commit_with_no_manifest_changes;
mod detail_lines_show_metadata_and_changes;
mod empty_history_has_no_selection;
mod navigation_wraps;

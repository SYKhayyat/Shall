pub use shall::app::adapters::*;
pub use shall::core::hook_lock::{adapter_id, hash_script, HookLedger};
use shall::core::LockFile;
pub use shall::model::layout::Layout;
use std::path::PathBuf;
use tempfile::TempDir;

fn layout_with(files: &[(&str, &str)]) -> (TempDir, Layout) {
    let dir = TempDir::new().unwrap();
    let adapters = dir.path().join("adapters");
    std::fs::create_dir_all(&adapters).unwrap();
    let locks = dir.path().join("locks");
    std::fs::create_dir_all(&locks).unwrap();
    let mut ledger = HookLedger::default();
    for (name, body) in files {
        let file = format!("{name}.toml");
        std::fs::write(adapters.join(&file), body).unwrap();
        ledger.approve(&adapter_id(&file), &hash_script(body));
    }
    ledger.save(&HookLedger::path_in(&locks)).unwrap();
    let layout = Layout::new(dir.path().to_path_buf(), dir.path().join("data"));
    (dir, layout)
}
fn standing(files: &[(&str, &str)], name: &str) -> Standing {
    let (_d, layout) = layout_with(files);
    survey(&layout)
        .into_iter()
        .find(|e| e.surface.name == name)
        .expect("every surface is surveyed")
        .standing
}
mod a_file_that_is_not_toml_names_the_place_it_broke;
mod a_file_the_ledger_has_never_seen_is_unapproved_and_is_not_parsed;
mod a_machine_that_has_extended_nothing_says_so_for_all_eight;
mod a_plural_row_header_is_reported_rather_than_silently_dropped;
mod a_row_of_the_right_kind_is_counted;
mod every_surface_is_named_once_and_resolves_by_name;
mod the_refusal_names_the_surface_the_file_and_how_a_row_opens;
mod the_row_key_being_a_table_rather_than_an_array_is_not_the_same_as_no_rows;

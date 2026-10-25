use serde::Deserialize;
pub use shall::core::adapter::AdapterRow;
pub use shall::model::bootstrap::*;

fn row(manager: &str, os: Option<&str>) -> BootstrapDef {
    BootstrapDef {
        manager: manager.into(),
        os: os.map(str::to_string),
        run: vec![
            "/bin/sh".into(),
            "-c".into(),
            format!("install {}", manager),
        ],
    }
}
mod a_manager_with_no_row_has_no_bootstrap;
mod a_row_for_this_os_beats_a_catch_all;
mod a_row_naming_an_os_applies_only_there;
mod a_row_without_an_os_applies_anywhere;
mod an_unusable_row_is_never_chosen;
mod the_command_is_argv_not_a_string_to_be_split;
mod the_command_line_is_shown_whole;

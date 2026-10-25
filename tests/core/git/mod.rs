pub use shall::core::git::*;
pub use shall::core::{Error, Result};
pub use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::tempdir;

fn git_or_skip() -> bool {
    static HERMETIC: std::sync::Once = std::sync::Once::new();
    // Twinned in `tests/mock_providers.rs`, which is a separate binary and cannot link to
    // this module. Change one, change the other.
    HERMETIC.call_once(|| {
        for (k, v) in [
            ("GIT_AUTHOR_NAME", "shall-tests"),
            ("GIT_AUTHOR_EMAIL", "test@example.invalid"),
            ("GIT_COMMITTER_NAME", "shall-tests"),
            ("GIT_COMMITTER_EMAIL", "test@example.invalid"),
            // Absent paths, so no `~/.gitconfig` or system config reaches these repos:
            // a host that signs every commit would otherwise fail them at `git commit`.
            ("GIT_CONFIG_GLOBAL", "shall-tests-absent-gitconfig"),
            ("GIT_CONFIG_SYSTEM", "shall-tests-absent-gitconfig"),
        ] {
            std::env::set_var(k, v);
        }
    });
    if !GitManager::git_available() {
        eprintln!("skipping: git not installed");
        return false;
    }
    true
}
mod a_missing_identity_is_named_as_the_reason_the_commit_failed;
mod a_signature_git_will_not_vouch_for_is_not_a_good_one;
mod checkout_files_restores_manifest_without_new_commit;
mod diff_manifest_changes_reports_package_level_delta;
mod git_is_told_not_to_ask_for_anything;
mod init_commit_head_and_log_round_trip;
mod init_writes_the_backup_glob_once_and_restores_a_commented_one;
mod parse_log_handles_subjects_with_spaces_and_separators;
mod parse_log_skips_blank_lines;
mod parse_manifest_changes_keeps_content_lines_drops_headers;

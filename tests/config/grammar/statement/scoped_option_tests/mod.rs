use super::*;

pub use shall::config::grammar::statement::*;
fn known(name: &str) -> bool {
    matches!(
        name,
        "apt" | "cargo" | "snap" | "btrfs" | "lvm" | "zfs" | "github"
    )
}
fn p(line: &str) -> Result<Statement> {
    parse(&Origin::new("modules/dev.txt", 3), line, &known)
}
fn opt(line: &str, key: &str) -> String {
    match p(line).unwrap_or_else(|e| panic!("`{}` was refused: {}", line, e)) {
        Statement::Package(d) => d.options.one(key).unwrap_or("").to_string(),
        other => panic!("expected a package, got {:?}", other),
    }
}

mod a_shim_can_be_asked_for_on_the_line_that_declares_the_tool;
mod a_snap_can_be_declared_unconfined;
mod a_storage_object_can_be_sized_and_mounted;
mod a_storage_option_is_refused_by_name_on_a_backend_that_cannot_read_it;
mod a_volume_can_be_given_the_size_lvm_requires;
mod a_volume_may_declare_that_it_is_allowed_to_shrink;
mod allow_shrink_without_a_size_is_refused;
mod mount_options_without_a_mount_is_refused;
mod the_storage_backends_do_not_share_each_others_options;

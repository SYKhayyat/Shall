use super::*;

#[test]
fn the_posture_table_covers_every_global_flag_clap_defines() {
    let globals = global_posture_flags();
    for a in <Cli as clap::CommandFactory>::command().get_arguments() {
        if !a.is_global_set() {
            continue;
        }
        if let Some(l) = a.get_long() {
            assert!(globals.contains(&format!("--{l}")), "--{l} is missing");
        }
        if let Some(c) = a.get_short() {
            assert!(globals.contains(&format!("-{c}")), "-{c} is missing");
        }
    }
    // And nothing that is not global leaked in.
    assert!(!globals.contains("--all"));
    assert!(!globals.contains("--locked"));
}

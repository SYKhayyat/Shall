use super::*;

/// Both nimble layouts, because the record one arrived in nimble 2 and the bare one is
/// still what older installs print.
#[test]
fn nimble_reads_a_version_out_of_either_layout() {
    let record = nimble_list(
        "nimpy  [(version: 0.2.1, checksum: 22173fb24ce9ca9d1c1db63fe15bdfb14e69c76a)]\n",
        "nimble",
    )
    .expect("nimble 2 output");
    assert_eq!(record[0].version.as_deref(), Some("0.2.1"));

    let bare = nimble_list("chronos  [1.0.0, 0.9.0]\n", "nimble").expect("nimble 0.13 output");
    assert_eq!(bare[0].version.as_deref(), Some("1.0.0"));
}

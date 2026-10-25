use super::*;

/// Real `btrfs filesystem show` output. The UUID is a token inside the first line, and the
/// code that read it wanted a line that *started* with `uuid:` — so it never found one, and
/// nothing noticed because its only caller was unreachable.
#[test]
fn the_filesystem_uuid_is_read_out_of_the_real_report() {
    let labelled = "Label: 'data'  uuid: 3b5f9c21-0f38-4a7c-9c2f-1b7d0a5e6c44\n\
                    \tTotal devices 1 FS bytes used 144.00KiB\n\
                    \tdevid    1 size 512.00MiB used 88.00MiB path /dev/loop0\n";
    assert_eq!(
        fs_uuid_in(labelled).as_deref(),
        Some("3b5f9c21-0f38-4a7c-9c2f-1b7d0a5e6c44")
    );
    // An unlabelled filesystem, which is what `mkfs.btrfs` with no `-L` produces.
    assert_eq!(
        fs_uuid_in("Label: none  uuid: 11111111-2222-3333-4444-555555555555\n").as_deref(),
        Some("11111111-2222-3333-4444-555555555555")
    );
    // A report with no UUID at all is a refusal, not an fstab entry saying `UUID=`.
    assert_eq!(fs_uuid_in("btrfs: command not found\n"), None);
    assert_eq!(fs_uuid_in(""), None);
}

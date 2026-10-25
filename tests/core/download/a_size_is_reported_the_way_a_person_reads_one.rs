use super::*;

#[test]
fn a_size_is_reported_the_way_a_person_reads_one() {
    assert_eq!(human_bytes(512), "512 B");
    assert_eq!(human_bytes(2 * 1024 * 1024 * 1024), "2.0 GiB");
    assert_eq!(human_bytes(DEFAULT_MAX_DOWNLOAD_BYTES), "2.0 GiB");
}

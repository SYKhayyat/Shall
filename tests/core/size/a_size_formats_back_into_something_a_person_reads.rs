use super::*;

#[test]
fn a_size_formats_back_into_something_a_person_reads() {
    assert_eq!(format_size(10 * (1 << 30)), "10G");
    assert_eq!(format_size(100 * (1 << 20)), "100M");
    assert_eq!(format_size(1536 * (1 << 20)), "1.50G");
    assert_eq!(format_size(512), "512B");
}

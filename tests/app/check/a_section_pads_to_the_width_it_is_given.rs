use super::*;

/// The summary is a column, and `Display` must honour the width it is given — `write_str`
/// ignores it, so the alignment silently does not happen.
#[test]
fn a_section_pads_to_the_width_it_is_given() {
    assert_eq!(format!("[{:<11}]", Section::Drift), "[drift      ]");
    assert_eq!(format!("[{:<11}]", Section::Unmanaged), "[unmanaged  ]");
}

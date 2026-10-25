use super::*;

#[test]
fn silence_on_both_axes_is_accepted() {
    assert!(linux64().accepts("fd.tar.gz"));
    assert!(linux64().accepts("fd"));
}

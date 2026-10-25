use super::*;

#[test]
fn nothing_kernel_shaped_is_no_kernel_change() {
    assert!(kernels_in(["jq", "nginx"].into_iter()).is_empty());
}

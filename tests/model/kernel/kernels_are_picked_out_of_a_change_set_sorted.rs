use super::*;

#[test]
fn kernels_are_picked_out_of_a_change_set_sorted() {
    let changed = ["jq", "linux-image-6.8", "nginx", "linux-headers-6.8", "jq"];
    assert_eq!(
        kernels_in(changed.into_iter()),
        vec![
            "linux-headers-6.8".to_string(),
            "linux-image-6.8".to_string()
        ]
    );
}

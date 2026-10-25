use super::*;

#[test]
fn a_param_gates_a_when_block() {
    // `$gpu` reaches a `when` — the same variable machinery, one scope wider.
    let f = fixture(&[(
        "workstation.txt",
        "param gpu = none\nwhen $gpu == nvidia {\n  apt:nvidia-driver\n}\n",
    )]);
    assert!(expand_with_args(&f, "workstation", &[]).unwrap().is_empty());
    let on = expand_with_args(&f, "workstation", &[("gpu", "nvidia")]).unwrap();
    assert_eq!(on.len(), 1);
}

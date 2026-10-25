use super::*;

#[test]
fn two_modules_declaring_one_package_differently_is_an_error_naming_both() {
    // Part IV requires this exact proof.
    let f = fx(
        "Work\n",
        &[("Work", "use a\nuse b\n")],
        &[
            ("a.txt", "apt:jq@version=1.6\n"),
            ("b.txt", "apt:jq@version=1.7\n"),
        ],
    );
    let err = resolve(&f).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("a.txt"), "{}", msg);
    assert!(msg.contains("b.txt"), "{}", msg);
}

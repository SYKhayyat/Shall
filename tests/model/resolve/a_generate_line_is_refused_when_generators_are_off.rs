use super::*;

#[test]
fn a_generate_line_is_refused_when_generators_are_off() {
    // U33/VI.0: a generator that reaches collection unexpanded (generators off) is a loud
    // refusal, never a silent skip — code the config meant to run did not.
    let f = fx(
        "Work\n",
        &[("Work", "use gen\n")],
        &[("gen.txt", "generate:./pick.sh\n")],
    );
    let err = resolve(&f).unwrap_err();
    assert!(err.what.contains("is not enabled"), "{}", err);
    assert!(
        err.hint.unwrap().contains("allow_generators"),
        "names the key"
    );
}

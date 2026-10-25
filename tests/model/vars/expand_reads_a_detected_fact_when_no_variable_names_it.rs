use super::*;

#[test]
fn expand_reads_a_detected_fact_when_no_variable_names_it() {
    // #69: `$home` in a `link:` value is the machine's answer, without a `vars` entry.
    let facts = expand_facts();
    assert_eq!(
        expand("topdirs = ${home}/Documents", &facts, &origin(1)).unwrap(),
        "topdirs = /home/u/Documents"
    );
    assert_eq!(
        expand("$user@$hostname", &facts, &origin(1)).unwrap(),
        "u@laptop"
    );
    assert_eq!(
        expand("on $os/$arch", &facts, &origin(1)).unwrap(),
        "on linux/x86_64"
    );
}

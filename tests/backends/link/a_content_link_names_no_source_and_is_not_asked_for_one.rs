use super::*;

/// Only the modes that open the file are subject to that refusal. `@content=` carries the
/// bytes in the line and never reads the name, so requiring the name to exist would refuse
/// a declaration that is complete.
#[test]
fn a_content_link_names_no_source_and_is_not_asked_for_one() {
    let mut opts = shall::config::grammar::Options::default();
    assert!(reads_its_source(&opts));
    opts.set("content", "hello\n".to_string());
    assert!(!reads_its_source(&opts));

    // A template and a secret both read their source, so both keep the check.
    let mut template = shall::config::grammar::Options::default();
    template.set("template", "true".to_string());
    assert!(reads_its_source(&template));
    let mut secret = shall::config::grammar::Options::default();
    secret.set("decrypt", "age".to_string());
    assert!(reads_its_source(&secret));
}

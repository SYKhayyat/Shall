use super::*;

#[test]
fn a_payload_option_is_recorded_as_a_digest_and_never_as_its_text() {
    let resources = resources_for(&shall::config::grammar::Statement::Link(
        "source".to_string(),
        {
            let mut opts = shall::config::grammar::Options::default();
            opts.set("target", "/etc/motd");
            opts.set("content", "the secret body");
            opts
        },
    ));
    let text = artifact(&resources);
    assert!(
        !text.contains("the secret body"),
        "a plan file a user leaves on disk carried the content it would write: {text}"
    );
    assert!(text.contains("content=#"), "{text}");
}

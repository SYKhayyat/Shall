use super::*;

/// **The container is what the judgement is made against, and `absent` is not `empty`.**
///
/// This is the case `or_unrecognised`'s JSON arm used to wave through: it returned
/// `Ok(found)` — empty — for *anything* whose output contained a parseable JSON document,
/// regardless of whether the reader had extracted a single package from it. So npm renaming
/// `dependencies`, or pip capitalising `name`, was a silently empty machine: install
/// everything, own nothing. Five backends reached that arm and were the unprotected ones,
/// while six sites elsewhere in the repo hand-rolled the correct rule.
#[test]
fn a_renamed_container_is_a_format_change_and_not_an_empty_machine() {
    // npm, with `dependencies` renamed. Valid JSON, parses perfectly, means nothing to us.
    let renamed = r#"{"name":"root","packages":{"typescript":{"version":"5.4.5"}}}"#;
    let err = parse_installed("npm", renamed).expect_err("a renamed container is a change");
    assert_eq!(err.backend, "npm");
    assert!(
        err.sample.contains("dependencies"),
        "the refusal must name the key it went looking for: {}",
        err.sample
    );

    // pnpm's array-of-objects shape, same rename.
    assert!(parse_installed("pnpm", &format!("[{renamed}]")).is_err());

    // pipx, with `venvs` renamed.
    assert!(parse_installed("pipx", r#"{"pipx_spec_version":"0.1","envs":{}}"#).is_err());

    // composer, with `installed` renamed — under its banner, so both fixes are in play.
    assert!(parse_installed(
        "composer",
        concat!(
            "Changed current directory to /root/.composer\n",
            r#"{"packages":[{"name":"psr/log","version":"1.1.4"}]}"#,
            "\n"
        )
    )
    .is_err());
}

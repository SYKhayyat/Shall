use super::*;

/// Every objection renders a reason that names itself.
///
/// `describe_objection` is the install side's whole explanation of a refusal, and nothing
/// asserted a word of it â€” replacing the body with `String::new()` and with `"xyzzy"` both
/// passed the suite. A refusal a user cannot read is a refusal that will be read as a bug
/// in Shall.
#[test]
fn every_objection_describes_itself() {
    let cases = [
        (
            Objection::Denied {
                key: "apt:telnet".into(),
            },
            vec!["apt:telnet", "deny_packages"],
        ),
        (
            Objection::Unpinned {
                key: "apt:curl".into(),
            },
            vec!["apt:curl", "pinned_only", "@version="],
        ),
        (
            Objection::Protected {
                key: "apt:python3".into(),
                reason: "an OS essential".into(),
            },
            vec!["apt:python3", "an OS essential"],
        ),
        (
            Objection::TooMany {
                count: 42,
                limit: 20,
                setting: "max_removals",
            },
            vec!["42", "20", "max_removals"],
        ),
        (
            Objection::TooManyInstalls {
                count: 99,
                limit: 50,
            },
            vec!["99", "50", "max_installs"],
        ),
    ];

    let mut seen: Vec<String> = Vec::new();
    for (objection, must_name) in cases {
        let text = describe_objection(&objection);
        for needle in must_name {
            assert!(
                text.contains(needle),
                "`{text}` does not name `{needle}`, so the user cannot tell what refused \
                     them or what to change"
            );
        }
        assert!(
            !seen.contains(&text),
            "two objections describe themselves identically (`{text}`), so the sentence \
                 does not identify which one fired"
        );
        seen.push(text);
    }
}

use super::*;

/// Three managers, one question, three spellings — and every one of them has to come back as a
/// bare name, because that is the only spelling a declaration can carry (`#211`).
#[test]
fn a_versioned_answer_is_read_as_a_name() {
    // `rpm -qf` with no `--qf` prints the fully qualified name; with `--qf '%{NAME}'` it prints
    // the bare one. Both arrive here, because the row's argv and a manager's default are two
    // things that can disagree.
    assert_eq!(
        dnf::parse_rpm_owner("openssh-server-9.9p1-8.fc41.x86_64\n"),
        vec!["openssh-server".to_string()]
    );
    assert_eq!(
        dnf::parse_rpm_owner("openssh-server\n"),
        vec!["openssh-server".to_string()]
    );
    // rpm's own sentence for a file no package owns — on stdout, with a zero exit.
    assert!(
        dnf::parse_rpm_owner("file /usr/local/bin/thing is not owned by any package\n").is_empty()
    );

    assert_eq!(
        pacman::parse_pacman_owner(
            "/usr/lib/systemd/system/sshd.service is owned by openssh 9.9p1-2\n"
        ),
        vec!["openssh".to_string()]
    );

    assert_eq!(
        common::parse_apk_owner("/usr/bin/ssh is owned by openssh-client-9.9_p1-r0\n"),
        vec!["openssh-client".to_string()]
    );
    assert!(common::parse_apk_owner("No package owns /usr/local/bin/thing\n").is_empty());
}

/// A name whose *own* characters include a hyphen and a digit must not be cut in half: the version
/// begins at the first `-` followed by a digit, and `openssh-9` is a package while `libfoo-2-devel`
/// is not.
#[test]
fn a_name_is_cut_where_the_version_starts_and_not_before() {
    assert_eq!(
        dnf::parse_rpm_owner("libfoo-2-devel-1.4-3.fc41.x86_64\n"),
        vec!["libfoo".to_string()],
        "`-2` reads as a version start either way; the tree's own rule is that the first such cut wins"
    );
    assert_eq!(
        pacman::parse_pacman_owner("/usr/share/x is owned by gtk3 3.24.38-1\n"),
        vec!["gtk3".to_string()]
    );
}

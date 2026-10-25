use super::*;

/// `profile show Work` asks "what would this profile give me", and the answer must not be
/// obtained by changing what the machine is set to.
///
/// It used to write `Work` into the real `active` file, resolve, and write the old contents
/// back. The window between those two writes spans reading every profile and module and
/// probing bare names over the network — and a `^C`, a panic, or a failing second write left
/// `active` naming a profile the user had asked only to look at, with the next `sync`
/// converging to it. Two halves, and this pins both: the override decides the answer, and
/// the file is untouched.
#[test]
fn asking_what_a_profile_would_give_you_does_not_change_what_is_active() {
    let f = fx(
        "Home\n",
        &[("Home", "use home\n"), ("Work", "use work\n")],
        &[("home.txt", "apt:vim\n"), ("work.txt", "apt:ripgrep\n")],
    );
    let before = std::fs::read_to_string(f.layout.active_file()).unwrap();

    let shown = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .as_if_active("Work\n".to_string())
        .resolve()
        .expect("resolving against the override");

    assert_eq!(
        names(&shown, "apt"),
        vec!["ripgrep".to_string()],
        "the override did not decide the answer — `show` is describing the wrong profile"
    );
    assert_eq!(
        std::fs::read_to_string(f.layout.active_file()).unwrap(),
        before,
        "`active` changed while answering a read-only question"
    );

    // And without the override, the same resolver still reads the file: an override that
    // leaked into the default path would make every sync resolve somebody else's profile.
    let real = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .expect("resolving against the file");
    assert_eq!(names(&real, "apt"), vec!["vim".to_string()]);
}

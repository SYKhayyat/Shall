use super::*;

#[test]
fn only_a_new_link_outside_home_is_asked_about() {
    #[cfg(windows)]
    let system = r"C:\ProgramData\shall\hosts";
    #[cfg(not(windows))]
    let system = "/etc/cron.d/backup";

    let state = state_with(vec![
        link("dotfiles/gitconfig", "~/.gitconfig"),
        link("cron/backup", system),
    ]);

    let asked = Dotfiles::outside_home_links(state.extras.iter(), &|_| false).unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, "link:cron/backup");
    assert_eq!(asked[0].1, std::path::PathBuf::from(system));

    // The destination is already there: it was agreed to on the run that placed it, and a
    // re-converge that asks again is a prompt on every sync.
    assert!(Dotfiles::outside_home_links(state.extras.iter(), &|_| true)
        .unwrap()
        .is_empty());
}

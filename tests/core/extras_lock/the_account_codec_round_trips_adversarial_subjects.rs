use super::*;

#[test]
fn the_account_codec_round_trips_adversarial_subjects() {
    let subjects = [
        "rg",
        "a@b",
        "a@@b",
        "@",
        "trailing@",
        "@leading",
        "org.gnome.desktop.theme/night-mode",
        "k@scope=system",
        "apt:ppa:x/y",
        "22/tcp",
        "with space",
        "org.gnome.x/theme@user",
        "unicode-Ω-日本語",
        "quote\"and'apostrophe",
    ];
    for subject in subjects {
        for user in [None, Some("alice"), Some("bob.smith-2"), Some("a")] {
            let key = ExtraKey::with_user(ResourceKind::Shim, subject, user).unwrap();
            let wire = key.to_string();
            let read: ExtraKey = wire
                .parse()
                .unwrap_or_else(|_| panic!("`{wire}` does not read back"));
            assert_eq!(read, key, "`{wire}` did not survive its own round trip");
            assert_eq!(read.to_string(), wire);
            assert_eq!(read.account_suffix(), user, "`{wire}` moved the account");
            assert_eq!(read.subject, subject, "`{wire}` did not keep the subject");
        }
    }
}

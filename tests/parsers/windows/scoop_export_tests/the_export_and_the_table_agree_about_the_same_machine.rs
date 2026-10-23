use super::*;

/// Both readers answer for the same machine, so they must answer alike — including about
/// the rows that are not packages.
#[test]
fn the_export_and_the_table_agree_about_the_same_machine() {
    let table = "Installed apps:\n\n\
                 Name     Version         Source Updated             Info\n\
                 ----     -------         ------ -------             ----\n\
                 7zip     26.00           main   2026-04-19 07:09:55     \n\
                 jq                              2026-07-21 13:48:29 Install failed\n\
                 halfway                         2026-07-21 13:48:29     \n\
                 fd       10.4.2          main   2026-07-08 15:15:49     \n";
    let a = parse_installed("scoop", table).expect("this fixture parses");
    let b = parse_scoop_export(EXPORT).expect("this fixture parses");
    assert_eq!(
        a.iter().map(|p| (&p.name, &p.version)).collect::<Vec<_>>(),
        b.iter().map(|p| (&p.name, &p.version)).collect::<Vec<_>>(),
    );
}

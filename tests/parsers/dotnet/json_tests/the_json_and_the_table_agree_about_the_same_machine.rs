use super::*;

/// The two forms describe the same machine, so they must report the same thing. A
/// difference here is a listing that changes shape with the installed SDK.
#[test]
fn the_json_and_the_table_agree_about_the_same_machine() {
    let table = "Package Id      Version      Commands \n\
                 --------------------------------------\n\
                 dotnetsay       3.0.3        dotnetsay\n";
    let a = parse_dotnet_list(table).expect("this fixture parses");
    let b = parse_dotnet_list_json(REAL).expect("this fixture parses");
    assert_eq!(
        a.iter().map(|p| (&p.name, &p.version)).collect::<Vec<_>>(),
        b.iter().map(|p| (&p.name, &p.version)).collect::<Vec<_>>(),
    );
}

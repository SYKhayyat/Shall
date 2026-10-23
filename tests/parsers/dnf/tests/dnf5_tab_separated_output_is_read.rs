use super::*;

#[test]
fn dnf5_tab_separated_output_is_read() {
    // Fedora 41 ships dnf5, which prints a tab and indents each row, and interleaves
    // "Matched fields:" headers. Reading only dnf4's ` : ` made dnf return nothing at
    // all on Fedora — so every unpinned name skipped the system manager and landed on
    // whichever language registry happened to publish the name.
    let input = "Updating and loading repositories:\nRepositories loaded.\n\
                 Matched fields: name (exact)\n \
                 jq.x86_64\tCommand-line JSON processor\n \
                 R-jqr.x86_64\tClient for 'jq', a 'JSON' Processor\n";
    let res = parse_dnf_search(input);
    assert_eq!(res.len(), 2, "headers must not become packages");
    assert_eq!(res[0].name, "jq");
    assert_eq!(res[1].name, "R-jqr");
    assert_eq!(
        res[0].properties.get("description").unwrap(),
        "Command-line JSON processor"
    );
}

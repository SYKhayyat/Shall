use super::*;

#[test]
fn placeholders_are_filled_left_to_right() {
    let template: Vec<String> = ["ufw", "allow", "{port}/{proto}"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        fill(&template, &[("{port}", "22"), ("{proto}", "tcp")]),
        vec!["ufw", "allow", "22/tcp"]
    );

    // A placeholder the row does not mention costs nothing — every caller passes the whole
    // vocabulary rather than branching per row.
    assert_eq!(
        fill(
            &template,
            &[("{policy}", "deny"), ("{port}", "22"), ("{proto}", "udp")]
        ),
        vec!["ufw", "allow", "22/udp"]
    );
}

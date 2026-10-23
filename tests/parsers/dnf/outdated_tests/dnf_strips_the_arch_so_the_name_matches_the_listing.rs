use super::*;

#[test]
fn dnf_strips_the_arch_so_the_name_matches_the_listing() {
    let p = parse_dnf_outdated(DNF);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "audit-libs");
    assert_eq!(p[0].version.as_deref(), Some("4.2.1-1.fc44"));
    assert!(
        !p.iter().any(|x| x.name.contains(".x86_64")),
        "an arch-qualified name matches nothing the caller holds: {:?}",
        p
    );
}

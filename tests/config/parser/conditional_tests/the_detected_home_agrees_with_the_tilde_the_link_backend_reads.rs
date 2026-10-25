use super::*;

#[test]
fn the_detected_home_agrees_with_the_tilde_the_link_backend_reads() {
    // One home, not two: `$home` in content and `~/` in a target must name the same
    // directory, and both go through `dirs`.
    let current = HostFacts::current();
    match (current.home, dirs::home_dir()) {
        (Some(fact), Some(dir)) => assert_eq!(
            std::path::PathBuf::from(fact),
            dir,
            "the fact and the target expansion disagree about home"
        ),
        (None, None) => {}
        (fact, dir) => panic!(
            "one home source answered and the other did not: {:?} vs {:?}",
            fact, dir
        ),
    }
}

use super::*;

/// The container's data directory is inside the container. A rehearsal that wrote a
/// registry to the host would make "it touched nothing" false.
#[test]
fn the_rehearsals_data_stays_in_the_container() {
    let cmd = argv("docker", "shall-it-ubuntu", "/cfg");
    let data = cmd
        .iter()
        .find(|a| a.starts_with("SHALL_DATA_DIR="))
        .expect("a data dir must be set");
    assert!(!data.contains("/cfg"), "{}", data);
    assert!(data.contains("/tmp/"), "{}", data);
}

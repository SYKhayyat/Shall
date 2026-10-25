use super::*;

/// `--rm`: a rehearsal that piled up stopped containers would be a command you stop running.
#[test]
fn the_container_removes_itself() {
    assert!(argv("docker", "i", "/cfg").iter().any(|a| a == "--rm"));
}

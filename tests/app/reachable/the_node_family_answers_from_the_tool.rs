use super::*;

/// `yarn` was cleared as "shims into a dir its own installer wires up". CI falsified that
/// on 2026-07-29: `yarn: cowsay is on PATH` FAILED on a clean Windows runner. The answer
/// cannot be a constant either — measured the same day, `yarn global bin` reads
/// `…\scoop\apps\yarn\current\global\bin` on the developer's box and `%LOCALAPPDATA%\Yarn\bin`
/// on the runner — so it is asked.
#[tokio::test]
async fn the_node_family_answers_from_the_tool() {
    let dir = Path::new(SANDBOX).join("yarn-global");
    let (exec, cfg) = mocked(&[("yarn global bin", &dir.display().to_string())]);
    assert_eq!(user_bin_dirs("yarn", &cfg, &exec).await, vec![dir]);
}

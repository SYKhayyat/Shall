use super::*;

#[tokio::test]
async fn the_seam_carries_what_the_active_profiles_reach() {
    let r = repo(&[
        ("priority", "apt\ncargo\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use editors\n"),
        ("modules/editors.txt", "apt:neovim\ncargo:ripgrep\n"),
        ("modules/gaming.txt", "apt:steam\n"),
    ]);
    let map = resolve(&r).await.unwrap();
    assert_eq!(names(&map, "apt"), ["neovim"]);
    assert_eq!(names(&map, "cargo"), ["ripgrep"]);
    // Nothing is active unless a profile names it: `gaming` was never reached.
    assert!(!names(&map, "apt").contains(&"steam".to_string()));
}

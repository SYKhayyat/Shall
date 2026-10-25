use super::*;

#[tokio::test]
async fn an_absent_line_crosses_the_seam_marked_absent() {
    // The map is the seam, so `absent:` shares it and carries `present: false`. The
    // planner splits them; nothing may read the map as a plain wish list.
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use base\n"),
        ("modules/base.txt", "apt:curl\nabsent:apt:libreoffice\n"),
    ]);
    let map = resolve(&r).await.unwrap();
    let apt = map.get("apt").unwrap();
    assert_eq!(names(&map, "apt"), ["curl"]);
    let absent: Vec<&str> = apt
        .iter()
        .filter(|s| !s.present)
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(absent, ["libreoffice"]);
}

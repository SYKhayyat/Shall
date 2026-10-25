use super::*;

#[test]
fn uninstall_reaches_every_module_the_active_profiles_hold() {
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use dev\n"),
        ("modules/dev.txt", "use base\napt:curl\n"),
        ("modules/base.txt", "apt:jq\n"),
        ("modules/unused.txt", "apt:steam\n"),
    ]);
    let files = active_module_files(&f.layout, &known, &facts());
    assert!(files.contains(&f.layout.module_file(&ModuleName::literal("dev"))));
    assert!(
        files.contains(&f.layout.module_file(&ModuleName::literal("base"))),
        "a module reached through another is still active"
    );
    assert!(
        !files.contains(&f.layout.module_file(&ModuleName::literal("unused"))),
        "nothing is active unless a profile reaches it"
    );
    assert!(
        files.contains(&f.layout.profile_file("Work")),
        "a profile may hold packages directly, so it is a file uninstall edits"
    );
}

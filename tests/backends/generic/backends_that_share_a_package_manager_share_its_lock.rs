use super::*;

/// Everything a manager prints that is *not* a dependency has to be thrown away, and until
/// 2026-07-30 none of it was: this parser took the first word of every non-empty line.
///
/// The first real `zypper` run in the project's history could not install a single package
/// because of it. `zypper info --requires jq` opens with `Loading repository data...` and
/// `Reading installed packages...`, and reports `Installed      : No` — so the dependency
/// list came back as `Loading`, `Reading`, `No` and a dozen other words. The planner adds
/// every dependency as an install node and then asks *it* for its dependencies, which
/// returned the same three words, and the sweep died on:
///
/// ```text
/// Error: `requires` forms a cycle
///   zypper:No requires zypper:Loading
///   zypper:Loading requires zypper:Reading
///   zypper:Reading requires zypper:No
/// ```
///
/// **Backends that drive one manager take one lock.** `pacman` and `yay` in one config is
/// an ordinary Arch machine, and both of them write `/var/lib/pacman/`. Keyed by their own
/// names they were two locks over one database: a sync touching both ran them at the same
/// time and let pacman's `db.lck` arbitrate, which it does by failing whichever lost.
///
/// A backend that shares its manager with nobody keys on itself, and that is checked too —
/// folding every backend onto one lock would serialise `npm` behind `apt` and throw away
/// the parallelism that makes a mixed sync worth running.
#[test]
fn backends_that_share_a_package_manager_share_its_lock() {
    let key = |name: &str| {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
        apt_like_core_named(name, exec).lock_key().to_string()
    };
    for (backend, expected) in [
        ("pacman", "pacman"),
        ("yay", "pacman"),
        ("paru", "pacman"),
        ("apt", "apt"),
        ("apt-get", "apt"),
        ("dnf", "dnf"),
        ("yum", "dnf"),
        ("microdnf", "dnf"),
        ("zypper", "zypper"),
        // No shared manager, no shared lock.
        ("npm", "npm"),
        ("cargo", "cargo"),
        ("flatpak", "flatpak"),
    ] {
        assert_eq!(key(backend), expected, "{backend} locks the wrong manager");
    }
}

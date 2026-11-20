use super::*;

#[test]
fn sandboxed_puts_every_path_under_the_sandbox() {
    // S11: the escape this exists to stop is a fixture that sets `config_root` and
    // forgets `data_root` -- the registry, journal and snapshots then land in the
    // developer's real state. Assert every path, not just the two obvious ones.
    let sandbox = std::path::Path::new(if cfg!(windows) {
        r"C:\shall-test-sandbox"
    } else {
        "/tmp/shall-test-sandbox"
    });
    let cfg = Config::sandboxed(sandbox);
    for (label, path) in [
        ("config_root", cfg.config_root()),
        ("data_root", cfg.data_root()),
        ("tmp_dir", cfg.tmp_dir.clone()),
        ("github_dir", cfg.github_dir.clone()),
        ("web_dir", cfg.web_dir.clone()),
        ("appimage_dir", cfg.appimage_dir.clone()),
        // `bin_dir` was the seventh, and it escaped: shims went to the real
        // `~/.local/bin` because the path was read from `dirs::home_dir()` at three call
        // sites instead of from here. A test run deployed an executable named `rg` onto
        // the developer's PATH.
        ("bin_dir", cfg.bin_dir.clone()),
    ] {
        assert!(
            path.starts_with(sandbox),
            "{} ({:?}) escaped the sandbox {:?}",
            label,
            path,
            sandbox
        );
    }
}

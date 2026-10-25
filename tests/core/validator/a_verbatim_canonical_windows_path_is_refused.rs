use super::*;

/// **The control was compiled dead on Windows.** `canonicalize()` answers a verbatim
/// `\\?\…` path there, `Path::starts_with` compares prefix *kinds* (`VerbatimDisk ≠
/// Disk`), and every entry in the list missed — including the registry hives, reachable
/// from Rhai's `read_file`. The verbatim spelling here is exactly what the real caller
/// hands over; it must meet the wall.
#[test]
fn a_verbatim_canonical_windows_path_is_refused() {
    let e = Validator::refuse_forbidden(Path::new(r"\\?\C:\Windows\System32\config\SAM"))
        .expect_err("the SAM hive is forbidden however the path is spelled");
    assert!(e.to_string().contains("SAM"), "{e}");
    assert!(
        Validator::refuse_forbidden(Path::new(r"\\?\C:\Windows\System32\config\SECURITY",))
            .is_err()
    );
}

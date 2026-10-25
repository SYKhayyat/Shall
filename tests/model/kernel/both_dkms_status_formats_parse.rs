use super::*;

/// Both `dkms status` formats are current, because distributions ship different DKMS
/// versions. Parsing only one would silently find no modules on half of them — and finding
/// no modules is indistinguishable from having none.
#[test]
fn both_dkms_status_formats_parse() {
    let old = "nvidia, 550.78, 6.8.0-31-generic, x86_64: installed";
    let new = "nvidia/550.78, 6.8.0-31-generic, x86_64: installed";
    for out in [old, new] {
        let mods = parse_status(out);
        assert_eq!(mods.len(), 1, "{:?} from {}", mods, out);
        assert_eq!(mods[0].name, "nvidia");
        assert_eq!(mods[0].version, "550.78");
        assert_eq!(mods[0].state, "installed");
        assert_eq!(mods[0].kernel, "6.8.0-31-generic");
    }
}

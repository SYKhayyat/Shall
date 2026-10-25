use super::*;

/// One module built for two kernels is two lines and two rows — the kernel is part of what
/// the line says, and collapsing it would hide "built for the old one, not the new one".
#[test]
fn several_modules_and_kernels_parse() {
    let out = "\
nvidia/550.78, 6.8.0-31-generic, x86_64: installed
nvidia/550.78, 6.8.0-35-generic, x86_64: installed
v4l2loopback/0.12.7, 6.8.0-31-generic, x86_64: installed
";
    let mods = parse_status(out);
    assert_eq!(mods.len(), 3, "{:?}", mods);
}

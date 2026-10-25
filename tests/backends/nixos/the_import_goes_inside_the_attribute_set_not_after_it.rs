use super::*;

/// **The defect a real NixOS caught and every hermetic test missed.** Appending the import
/// after the closing brace puts it outside the attribute set, and nix refuses the file with
/// `syntax error, unexpected '='` — which breaks the machine's boot configuration, not just
/// Shall. These pin the two shapes that are safe to edit.
#[test]
fn the_import_goes_inside_the_attribute_set_not_after_it() {
    let plain = "{ config, pkgs, ... }:
{
  system.stateVersion = \"26.05\";
}
";
    let out = with_import(plain, GENERATED).expect("a plain module is editable");
    assert!(imports_generated(&out), "{out}");
    // Inside: the import must come before the closing brace, not after it.
    let imp = out.find(GENERATED).expect("import");
    let close = out.rfind('}').expect("closing brace");
    assert!(
        imp < close,
        "the import landed outside the attribute set:
{out}"
    );
    // And not inside the ARGUMENT pattern, which is the other brace in the file.
    let pattern_end = out.find("}:").expect("argument pattern");
    assert!(
        imp > pattern_end,
        "the import landed in the argument list:
{out}"
    );
}

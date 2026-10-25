use super::*;

/// Every edited shape is written out for `nix-instantiate --parse`, exactly as the rendered
/// ones are. The generated file was already covered; the file Shall *edits* was not, which
/// is precisely where the defect was.
#[test]
fn every_edited_configuration_is_written_for_nix_to_parse() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/nix-fixtures");
    std::fs::create_dir_all(&dir).expect("fixture dir");
    let cases = [
        (
            "edited-plain",
            "{ config, pkgs, ... }:
{
  system.stateVersion = \"26.05\";
}
",
        ),
        (
            "edited-with-imports",
            "{ config, pkgs, ... }:
{
  imports = [ ./hardware-configuration.nix ];
  system.stateVersion = \"26.05\";
}
",
        ),
    ];
    for (name, body) in &cases {
        let edited = with_import(body, GENERATED).expect("editable");
        std::fs::write(dir.join(format!("{name}.nix")), edited).expect("write");
    }
}

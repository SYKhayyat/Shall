//! The quiet one, stated separately because the count above would still pass if `binary` were
//! never chosen at all: across every real asset here, the only files Shall would run as
//! executables are the ones whose names say which machine they are for.

use super::*;

#[test]
fn nothing_becomes_an_executable_without_naming_the_machine() {
    for b in blocks() {
        for (os, arch, _) in &b.expected {
            let platform = Platform::new(os.clone(), arch.clone());
            for asset in &b.assets {
                if Format::is_metadata_filename(&asset.name) {
                    continue;
                }
                if classify_format(&asset.name, &platform, None) == Some(Format::Binary) {
                    assert!(
                        platform.specificity(&asset.name) > 0,
                        "{} would be run as an executable on {}/{} while naming neither",
                        asset.name,
                        os,
                        arch
                    );
                }
            }
        }
    }
}

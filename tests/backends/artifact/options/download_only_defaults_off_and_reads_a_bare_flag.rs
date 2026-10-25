use super::*;

#[test]
fn download_only_defaults_off_and_reads_a_bare_flag() {
    assert!(!ArtifactOptions::read(&opts(&[])).unwrap().download_only);
    // The grammar turns a bare `@download_only` into "true".
    assert!(
        ArtifactOptions::read(&opts(&[("download_only", "true")]))
            .unwrap()
            .download_only
    );
    assert!(
        !ArtifactOptions::read(&opts(&[("download_only", "false")]))
            .unwrap()
            .download_only
    );
    assert!(
        !ArtifactOptions::read(&opts(&[("download_only", "no")]))
            .unwrap()
            .download_only
    );
}

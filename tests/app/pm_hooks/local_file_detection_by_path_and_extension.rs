use super::*;

#[test]
fn local_file_detection_by_path_and_extension() {
    assert_eq!(
        classify_install_target("./htop.deb"),
        InstallKind::LocalFile
    );
    assert_eq!(
        classify_install_target("/tmp/x.rpm"),
        InstallKind::LocalFile
    );
    assert_eq!(
        classify_install_target("C:\\pkgs\\a.msi"),
        InstallKind::LocalFile
    );
    assert_eq!(
        classify_install_target("foo-1.0.pkg.tar.zst"),
        InstallKind::LocalFile
    );
    assert_eq!(classify_install_target("ripgrep"), InstallKind::Repo);
    assert_eq!(classify_install_target("apt:curl"), InstallKind::Repo);
}

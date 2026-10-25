use super::*;

#[test]
fn the_kernels_of_every_distribution_are_recognised() {
    for name in [
        "linux",
        "linux-lts",
        "linux-zen",
        "linux-hardened",
        "linux-image-6.8.0-31-generic",
        "linux-headers-amd64",
        "linux-generic",
        "kernel",
        "kernel-core",
        "kernel-default",
        "kernel-devel",
    ] {
        assert!(is_kernel_package(name), "`{}` is a kernel package", name);
    }
}

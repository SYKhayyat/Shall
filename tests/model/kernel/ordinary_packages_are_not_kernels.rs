use super::*;

/// A false negative costs a driver, so the matcher is generous — but a false positive can
/// fail an unrelated sync on a pre-existing stuck module, so these near-misses must NOT
/// trip it. Every one is a real package that looks kernel-shaped and is not.
#[test]
fn ordinary_packages_are_not_kernels() {
    for name in [
        "jq",
        "ripgrep",
        "linuxbrew",
        "vlinux",
        "nginx",
        "python3",
        "linux-libc-dev", // userspace C headers, not a kernel
        "linux-tools-common",
        "kernel-tools", // the one the bare `kernel-` prefix used to catch
        "kernel-rpm-macros",
        "kernelshark",
        "liblinux",
    ] {
        assert!(!is_kernel_package(name), "`{}` is not a kernel", name);
    }
}

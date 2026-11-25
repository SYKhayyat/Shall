{ pkgs, ... }:

# The NixOS machine `nix/activation.nix` boots — and, because it is also copied in as
# `/etc/nixos/shall-vm-base.nix`, the exact configuration `nixos-rebuild switch` re-reads.
#
# **Nothing here may name a path outside this file.** The rebuild evaluates a *copy in the store*,
# so `./package.nix` or `../Cargo.lock` would resolve beside a store path that does not have them,
# and the switch would fail on a missing file rather than on anything Shall did. `shall` itself is
# installed by the test node and copied to a fixed path there for the same reason.
#
# **That identity is what makes the switch a real test rather than an experiment.** The rebuild
# inside the VM evaluates this same file, so the only difference between the running system and the
# one it builds is `imports = [ ./shall-packages.nix ]` — the file Shall writes. A rebuild from a
# *different* configuration would drag in a bootloader, hardware modules and a channel the VM does
# not have, and fail for reasons that say nothing about Shall (the failure mode `src/backends/
# nixos.rs` records: 45.9 s spent rebuilding somebody else's system and exiting 0).
{
  # **The two module sets a test node has implicitly and a rebuilt system does not.** `qemu-vm.nix`
  # is what gives the VM its console, disk and 9p store, and `test-instrumentation.nix` is what the
  # driver talks to; a switch to a system without them succeeds and then loses the machine the test
  # is running in. Importing them here is what makes "the configuration the rebuild re-reads" the
  # same *machine* and not merely the same hostname.
  imports = [
    <nixpkgs/nixos/modules/virtualisation/qemu-vm.nix>
    <nixpkgs/nixos/modules/testing/test-instrumentation.nix>
  ];

  system.stateVersion = "26.05";

  # **The root filesystem, stated rather than generated.** `nix-build` refuses a system whose
  # `fileSystems` names no root, and this is the value the test VM actually runs on — measured
  # inside it (`findmnt -no SOURCE,FSTYPE /` prints `/dev/vda ext4`) rather than copied from a
  # normal machine. `switch-to-configuration` therefore sees an unchanged fstab, which is what keeps
  # a rebuild an *activation* and not a re-partition.
  fileSystems."/" = {
    device = "/dev/vda";
    fsType = "ext4";
  };
  swapDevices = [ ];

  # Pin nixpkgs into the store the VM already has: `nixos-rebuild` evaluates `<nixpkgs/nixos>`, and
  # a VM test has no network. `utils.toShellPath`-style quoting is not needed because the channel is
  # one path.
  nix.nixPath = [ "nixpkgs=${pkgs.path}" ];
  nix.settings = {
    # The VM has no user namespaces, and nothing here may be fetched: a rebuild that reached the
    # network would pass on a connected runner and fail on a capped one, which is the shape of test
    # this repo refuses.
    sandbox = false;
    substituters = [ ];
    experimental-features = [ "nix-command" ];
  };

  # In the closure but **not** installed: `shall apply nixos:hello` makes the rebuilt system
  # reference it, and `nixos-rebuild` must find it in the store rather than substitute it.
  system.extraDependencies = [ pkgs.hello ];

  # A test VM has no firmware and no disk to put a bootloader on, and `nixos-rebuild switch` fails
  # at the install step for that reason alone before it ever reaches the generation swap.
  boot.loader.grub.enable = false;
  boot.loader.systemd-boot.enable = false;

  documentation.enable = false;
  services.getty.autologinUser = "root";
}

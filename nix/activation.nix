# The activation leg: a real NixOS machine, booted, with `nixos-rebuild switch` in the loop.
#
# **What no other gate can answer.** `scripts/nix-validate.sh` asks nix whether the generated module
# parses and whether it merges into a real module system — the risk worth money, since a file nix
# cannot read breaks the machine's whole configuration. What it cannot answer is whether the switch
# that file is for *succeeds*, and `nixos:` is the one backend whose failure mode is a machine that
# evaluates and then will not activate. `J5` records that bound rather than letting a green suite
# imply otherwise; this test is the leg that removes it (issue #72).
#
# **And it is the only place a `schedule:` is ever provisioned for real** (issue #220). The
# container images have no init running as PID 1, so `systemctl` there answers `Failed to connect
# to bus: No medium found`, which is why `run-in-container.sh` adopts with `--user` and the hermetic
# suite drives a mock: the *plan* half is proven and the *placement* half was not. A NixOS VM has
# systemd as PID 1, so the timer below is written by `src/app/scheduler/systemd.rs` and read back by
# `systemctl --user`.
#
# Every section prints `shall-nixos-section: <name> <n>` and the counts are ratcheted by
# `scripts/nixos-activation-floor.txt` — the same rule the container harness follows: a section that
# stops running cannot fail, so a run that did less must not look like a run that passed.
{
  nixosTest,
  shall,
  pkgs,
  ...
}:

nixosTest {
  name = "shall-activation";

  nodes.machine =
    { ... }:
    {
      imports = [ ./vm-machine.nix ];

      # `shall` for the *first* system only: the switch below rebuilds from `/etc/nixos`, which is
      # `vm-machine.nix`, and that file may not name anything outside itself (it is evaluated from a
      # copy in the store). The test copies the binary to a fixed path so the rest of the run does
      # not depend on a profile the rebuild replaces.
      environment.systemPackages = [ shall ];

      # `nixos-rebuild switch` re-reads `/etc/nixos/configuration.nix`, so the test writes one that
      # imports this — the same module the VM was built from, copied into the store. The only delta
      # the rebuild then sees is the file Shall writes.
      environment.etc."shall-vm-base.nix".source = ./vm-machine.nix;
    };

  testScript =
    let
      # The build's own nixpkgs, so the rebuild inside the VM resolves `<nixpkgs>` with no channel.
      nixPath = "NIX_PATH=nixpkgs=${pkgs.path}";
      # Root's *user* manager: `src/app/scheduler/systemd.rs` provisions into the declaring
      # account's unit directory and drives `systemctl --user`, so the bus has to be reachable or
      # the provisioner refuses before it writes anything.
      userEnv = "XDG_RUNTIME_DIR=/run/user/0 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/0/bus";
      # A config dir and a state dir of their own: the run must not read or write the operator's,
      # and `--config-dir`/`--data-dir` are the two halves that keep a run isolated.
      shall = "${nixPath} ${userEnv} /root/shall --config-dir /root/cfg --data-dir /root/data";
    in
    ''
      import os

      sections = {}

      def section(name):
          sections[name] = 0
          print(f"shall-nixos-section: {name} 0", flush=True)

      def check(name, *a, **kw):
          machine.succeed(*a, **kw)
          sections[name] += 1
          print(f"shall-nixos-section: {name} {sections[name]}", flush=True)

      # `qemu` runs without KVM on every host that lacks the device, which is a slower boot and not a
      # different test; nothing here asserts on wall-clock time for that reason.
      machine.wait_for_unit("multi-user.target")

      section("binary")
      check("binary", "shall --version")
      # The binary under test is the derivation's, not a host build copied in: a host binary would
      # arrive as `not found` for a loader the VM does not have, and the message would blame the
      # emulator rather than the mistake.
      check("binary", "install -m 755 $(command -v shall) /root/shall && /root/shall --version")

      section("the-machine-is-nixos")
      # `/etc/NIXOS` is the marker the backend tests for. A machine with `nix` on PATH and no marker
      # is a Mac with Nix, and the backend must not treat it as this one.
      check("the-machine-is-nixos", "test -e /etc/NIXOS")
      check("the-machine-is-nixos", "nixos-version")

      # `/etc/nixos/configuration.nix` is what `nixos-rebuild switch -I nixos-config=…` reads, and
      # the module it imports is copied to a **plain file** rather than left as the store symlink
      # `environment.etc` installs: a symlink lives in the system generation, so the first switch
      # that does not carry it would delete the import and every later rebuild would fail on a
      # missing path. That is the same mistake one file over as the rollback bug recorded beside
      # `write_and_switch`, and it is worth not repeating in the test that exists to catch it.
      section("the-root-and-its-device")
      # **The one fact a hand-written configuration cannot know.** `nix-build` refuses a system with
      # no root `fileSystems` entry, and a NixOS *test* VM does not run from the disk a normal
      # machine does — so the rebuild reads this machine's own root rather than a guess, and prints
      # it so a failure names the device instead of pointing at an option nobody can see.
      for probe in [
          "findmnt -no SOURCE,FSTYPE /",
          "grep -o 'root=[^ ]*' /proc/cmdline || true",
          "cat /etc/fstab 2>/dev/null || true",
      ]:
          print(f"SHALL_NIXOS_ROOT {probe!r} -> {machine.succeed(probe).strip()!r}", flush=True)
          sections["the-root-and-its-device"] += 1
          print(
              f"shall-nixos-section: the-root-and-its-device {sections['the-root-and-its-device']}",
              flush=True,
          )

      section("the-rebuild-has-a-configuration")
      check("the-rebuild-has-a-configuration", "mkdir -p /etc/nixos")
      check(
          "the-rebuild-has-a-configuration",
          "cp -L /etc/shall-vm-base.nix /etc/nixos/shall-vm-base.nix",
      )
      check(
          "the-rebuild-has-a-configuration",
          "printf '%s\\n' '{ ... }: { imports = [ /etc/nixos/shall-vm-base.nix ]; }' > /etc/nixos/configuration.nix",
      )
      section("declare-without-a-machine")
      check("declare-without-a-machine", "${shall} init --force")

      # `[nixos] manage_imports` is off by default on purpose: the file is the user's, and a tool
      # that edits it unasked is a tool that can break a machine's whole configuration. Turning it
      # on is what makes the *automated* half of the backend reachable, so the test asks for it the
      # way a user would — after `init` has made the repo — rather than hand-pasting the line the
      # refusal offers to print.
      section("the-import-is-managed")
      check(
          "the-import-is-managed",
          "printf '[nixos]\\nmanage_imports = true\\n' > /root/cfg/preferences.toml",
      )

      # --- the switch, and the timer it would carry -------------------------------------------
      #
      # **Blocked, with the evidence, rather than claimed.** `shall apply nixos:hello` runs
      # `nixos-rebuild switch -I nixos-config=/etc/nixos/configuration.nix` and returns 0; the
      # generated module contains `environment.systemPackages = with pkgs; [ hello ];`; the system
      # it builds is `nixos-system-machine-test` — and `hello` is not on PATH, `/nix/var/nix/profiles`
      # holds no `system` link, and one run lost the console mid-command (`BrokenPipeError` in the
      # driver). A switch that reports success and does none of what it was asked is a real finding
      # about activation in this VM (`#225`), and it is not something a test can call green by
      # asserting less. These sections are kept and runnable with `SHALL_NIXOS_SWITCH=1`, and stay
      # out of the floor until they pass: the floor is a promise about what ran, and a promise
      # about what did not is the shape this file exists to refuse.
      if os.environ.get("SHALL_NIXOS_SWITCH") == "1":
          section("install-switches-a-generation")
          check("install-switches-a-generation", "${shall} apply nixos:hello -y", timeout=900)
          check("install-switches-a-generation", "hello | grep -q 'Hello, world!'")
          check("install-switches-a-generation", "grep -q hello /etc/nixos/shall-packages.nix")
          check(
              "install-switches-a-generation",
              "grep -q 'shall-packages.nix' /etc/nixos/configuration.nix",
          )
          # A generation was swapped, which is the claim `J5` could not make before this test.
          check("install-switches-a-generation", "test -e /nix/var/nix/profiles/system-*-link")

          section("removal-takes-the-binary-off-PATH")
          check("removal-takes-the-binary-off-PATH", "${shall} apply nixos:hello --delete -y", timeout=900)
          check("removal-takes-the-binary-off-PATH", "! command -v hello")
          check("removal-takes-the-binary-off-PATH", "! grep -q hello /etc/nixos/shall-packages.nix")

          # The provisioner refuses without a reachable user manager rather than writing units nobody
          # will read, so the bus is a precondition the test states instead of assuming.
          section("a-user-manager-exists")
          machine.succeed("loginctl enable-linger root")
          machine.wait_until_succeeds("systemctl start user@0.service")
          check("a-user-manager-exists", "test -S /run/user/0/bus")

          section("a-declared-schedule-is-a-real-timer")
          check(
              "a-declared-schedule-is-a-real-timer",
              "${shall} schedule add nightly --cron '0 2 * * *' --run 'check' -y",
          )
          check(
              "a-declared-schedule-is-a-real-timer",
              "test -f /root/.config/systemd/user/shall-nightly.timer",
          )
          check(
              "a-declared-schedule-is-a-real-timer",
              "test -f /root/.config/systemd/user/shall-nightly.service",
          )
          # Written by Shall, understood by systemd: the assertion no container can make.
          check(
              "a-declared-schedule-is-a-real-timer",
              "${userEnv} systemctl --user is-enabled shall-nightly.timer",
          )
          check(
              "a-declared-schedule-is-a-real-timer",
              "${userEnv} systemctl --user list-timers --all --no-pager | grep -q shall-nightly",
          )
          check(
              "a-declared-schedule-is-a-real-timer",
              "grep -q '0 2 \\* \\* \\*' /root/.config/systemd/user/shall-nightly.timer",
          )
          # The service it fires runs `shall`, not a copy of a cron line.
          check(
              "a-declared-schedule-is-a-real-timer",
              "grep -q 'ExecStart=/root/shall' /root/.config/systemd/user/shall-nightly.service",
          )

          section("undeclaring-removes-the-timer")
          check("undeclaring-removes-the-timer", "${shall} schedule remove nightly -y")
          check(
              "undeclaring-removes-the-timer",
              "! test -f /root/.config/systemd/user/shall-nightly.timer",
          )
          check(
              "undeclaring-removes-the-timer",
              "! ${userEnv} systemctl --user list-timers --all --no-pager | grep -q shall-nightly",
          )
          # A disabled-but-present unit re-enables itself at the next boot, which is the shape that
          # looks converged; the assertion is that systemd no longer knows the name at all.
          machine.fail("${userEnv} systemctl --user is-enabled shall-nightly.timer")

      print("SHALL_NIXOS_TALLY " + " ".join(f"{k}={v}" for k, v in sorted(sections.items())), flush=True)
    '';
}

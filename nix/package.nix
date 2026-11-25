{ lib, rustPlatform, pkg-config }:

# Shall, for the one thing the tree cannot do without it: putting a real `shall` binary inside a
# NixOS VM so `nixos-rebuild switch` can be driven instead of argv-checked (see `nix/activation.nix`
# and issue #72).
#
# **Why a real derivation rather than the host's `target/release/shall`.** A VM built from the same
# nixpkgs shares no store paths with a binary linked against this host's glibc by name, so copying
# the host binary in is a coin flip that fails as `/lib64/ld-linux-x86-64.so.2: not found` — a
# message pointing at the emulator rather than at the mistake. Building it here means the closure
# the VM needs is the closure nix already knows about.
#
# `doCheck = false`: this builds the *binary*, and the suite is CI's job. `cargo test` here would
# spawn processes, want a writable data dir and take twenty minutes inside a sandbox that has no
# business giving it one — a packaging step that measures the test suite is measuring the wrong
# thing. The gates that matter are `cargo test --no-fail-fast`, the container harness, and the
# activation test this file exists for.
rustPlatform.buildRustPackage {
  pname = "shall";
  version = (lib.importTOML ../Cargo.toml).package.version;

  # `target/` is 113 GB of build artifacts and `.git` is history nobody is going to compile;
  # without this filter nix copies both into the store before it starts.
  #
  # **The non-Rust trees are excluded for a second reason: the source hash.** Every file under
  # `src` is in the hash, so editing `nix/activation.nix` while iterating on the test rebuilt the
  # whole 448-crate binary — twice in one afternoon — and nothing about `docs/`, `scripts/`,
  # `docker/` or `.github/` reaches the crate (every `include_str!` in `src/` names a `.toml`
  # beside it, which the check `include_str!` leaves below pins).
  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      let
        base = baseNameOf (toString path);
      in
      !(builtins.elem base [
        "target"
        ".git"
        "result"
        ".comtai"
        "nix"
        "docs"
        "scripts"
        "docker"
        ".github"
        "images"
      ]);
  };

  cargoLock.lockFile = ../Cargo.lock;

  # `mlua`'s vendored Lua, `xz2`'s and `zstd`'s vendored C are compiled by `cc` against the
  # sandbox's own compiler; nothing here looks for a system library (the TLS stack is rustls, so
  # there is no `openssl` to find and no `openssl-src` to build).
  nativeBuildInputs = [ pkg-config ];

  doCheck = false;

  meta = {
    description = "Declare a machine - packages, services, settings, firewall, dotfiles - and sync makes it match";
    homepage = "https://github.com/SYKhayyat/Shall";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "shall";
  };
}

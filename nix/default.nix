# `nix build -f nix` gives `./result/bin/shall`; `nix build -f nix activation-test` boots a NixOS VM
# and drives `nixos-rebuild switch` through it (issue #72). Everything here is `import <nixpkgs>`
# rather than a flake lock, because the rest of this repo's Nix usage (`scripts/nix-validate.sh`)
# reads the channel too and a lock file would be a second answer to "which nixpkgs".
{ nixpkgs ? import <nixpkgs> { } }:

let
  pkgs = nixpkgs;
  shall = pkgs.callPackage ./package.nix { };
in
{
  inherit shall;
  activation-test = pkgs.callPackage ./activation.nix { nixosTest = pkgs.testers.nixosTest; inherit shall; };
}

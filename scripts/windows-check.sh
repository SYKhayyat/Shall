#!/usr/bin/env bash
# Compile — and optionally run — the Windows half of the tree from a Linux host.
#
# `scripts/unix-check.sh` is the other half: the local chain's other four steps verify Linux (and
# the host is not Linux for the cfg blocks they cannot see), and the Windows leg of CI was red for
# fourteen errors across three files that nothing local could see. `cargo check --target
# x86_64-pc-windows-gnu` closes the compile half; wine closes the run half, because a test that
# passes on Linux and fails on the Windows runner is usually a *behaviour* difference (`Add-Content`
# losing a write to another handle) and not a type error.
#
# Usage:
#   scripts/windows-check.sh                 # cargo check, -D warnings, as ci.yml spells it
#   scripts/windows-check.sh --run <filter>  # build the suite for the target and run it under wine
#
# What it needs, and why the shim exists (each of these failed as a code defect before it was
# understood):
#   * the mingw C toolchain, because mlua/xz2/zstd/zip vendor C — `cargo check` needs one for the
#     *target* even though it does not link;
#   * `libpthread.a`, which `pkgsCross.mingwW64.stdenv.cc` does not carry and which rustc asks for
#     as `-l:libpthread.a` the moment anything links. Nix's cc-wrapper ignores `LIBRARY_PATH`, so
#     the search path rides on the linker itself — as a shim named what the linker is already
#     called, not as `RUSTFLAGS`, which cargo fingerprints and which would rebuild all 448 crates;
#   * `libz.so.1` for the *rustup* `rustc`, which is where the cross `rust-std` lives;
#   * PowerShell, which wine does not have. Every hook test whose script is `#!...powershell` fails
#     under `--run` with an absent interpreter, and that is the emulator, not the code — those pass
#     on the real Windows runner, which is what this is a rehearsal for.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
TOOLCHAIN="${SHALL_RUSTUP_TOOLCHAIN:-$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu}"
CACHE=${SHALL_WINDOWS_CACHE:-/tmp/shall-windows-check}

RUN_FILTER=""
if [ "${1:-}" = "--run" ]; then
  RUN_FILTER=${2:?--run needs a test-name filter}
fi

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "windows-check: $1 is not on PATH, and this gate cannot run without it" >&2
    exit 2
  }
}
need nix
need cargo
[ -x "$TOOLCHAIN/bin/rustc" ] || {
  echo "windows-check: no rustup toolchain at $TOOLCHAIN (set SHALL_RUSTUP_TOOLCHAIN)" >&2
  exit 2
}

mkdir -p "$CACHE"

# The cross C compiler, the pthreads library it does not ship, and zlib for the rustup rustc.
[ -x "$CACHE/mingw-cc/bin/x86_64-w64-mingw32-cc" ] ||
  nix build --impure --expr 'let p = import <nixpkgs> {}; in p.pkgsCross.mingwW64.stdenv.cc' \
    --out-link "$CACHE/mingw-cc"
[ -d "$CACHE/mingw-pthreads/lib" ] ||
  nix build --impure --expr \
    'let p = import <nixpkgs> {}; in p.pkgsCross.mingwW64.windows.mingw_w64_pthreads' \
    --out-link "$CACHE/mingw-pthreads"
[ -d "$CACHE/zlib/lib" ] || nix build nixpkgs#zlib --out-link "$CACHE/zlib"

# The linker, with the pthreads search path attached. Both names: cargo links with `-cc`, cc-rs
# compiles vendored C with `-gcc`.
mkdir -p "$CACHE/shim"
for name in x86_64-w64-mingw32-cc x86_64-w64-mingw32-gcc; do
  printf '#!/bin/sh\nexec %s/bin/%s "$@" -L%s/lib\n' \
    "$CACHE/mingw-cc" "$name" "$CACHE/mingw-pthreads" >"$CACHE/shim/$name"
  chmod +x "$CACHE/shim/$name"
done

export PATH="$TOOLCHAIN/bin:$CACHE/shim:$CACHE/mingw-cc/bin:$PATH"
export LD_LIBRARY_PATH="$CACHE/zlib/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-cc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-c++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

cd "$ROOT"

if [ -z "$RUN_FILTER" ]; then
  # `-D warnings` rides in `RUSTFLAGS` because `cargo check` takes no trailing rustc arguments, and
  # it is not optional: two of the fourteen errors this gate exists for were `unused_variables`, a
  # warning locally and fatal under the board's gate.
  RUSTFLAGS="-D warnings" cargo check --target x86_64-pc-windows-gnu --all-targets --all-features \
    --locked
  echo "windows-check: compiles clean for x86_64-pc-windows-gnu"
  exit 0
fi

need nix
cargo test --no-run --target x86_64-pc-windows-gnu --test suite
exe=$(ls -t target/x86_64-pc-windows-gnu/debug/deps/suite-*.exe | head -1)
echo "windows-check: running $exe under wine (filter: $RUN_FILTER)"
# One thread: wine serialises these anyway, and two hooks in one test writing one file is the
# mistake this gate was built to catch, not to reproduce.
WINEDEBUG=-all WINEDLLOVERRIDES="mscoree,mshtml=" \
  WINEPATH="$CACHE/mingw-cc/bin;$CACHE/mingw-pthreads/bin" \
  nix shell nixpkgs#wine64 --command wine "$exe" --test-threads 1 "$RUN_FILTER"

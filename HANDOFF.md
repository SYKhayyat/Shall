# HANDOFF — the #120 test-layout sweep, as of 2026-10-23

**The next session should start by reading this file and then doing exactly one
thing: the `executor.rs` move.** Everything needed is here; the state is green and
committed.

---

## 1. Where the tree stands

`main` is green and pushed. The verify chain passes on the committed tree:

| step | result |
|---|---|
| `cargo build --all-targets` | clean |
| `cargo test --no-fail-fast` | 1,932 lib + 800 suite, none failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | clean |
| `cargo fmt -- --check` (rustfmt **1.97.1**) | clean |
| `cargo check --target x86_64-pc-windows-gnu --lib --tests` | clean |

**Closed this session (5 issues):** #116 (the gate only credits a catch that
reproduces — nextest retries), #121 (the credential answer belongs to the run),
#118 (the lock tells a busy lock from a broken one), #119 (the worker shutdown and
the lock budget), #122 (the guard's second capped list).

**Open, and the sweep's own subjects:** #120 (this sweep), #117 (six of eleven
sync-engine subjects are covered; the rest need the machine fixture).

## 2. The acceptance criteria

A batch of files is done only when **all ten** hold. They are written as the tool's
own test — `python3 scripts/move-inline-tests_test.py` — so a broken tool fails
before a broken tree does.

1. `cargo build --all-targets` — clean.
2. `cargo test --no-fail-fast` — none failed.
3. **No test is lost** — the lib target's count falls by exactly the tests that
   moved; the suite's rises by the same number. Measured with a test-count script.
4. Clippy, `-D warnings`, `--all-features` — no leftover unused imports.
5. `cargo fmt -- --check` — rustfmt 1.97.1, which is what CI pins.
6. `every_test_file_is_in_the_suite` — every new file is declared.
7. The scanning gates: `a_module_is_a_subject_not_a_pile`,
   `no_comment_cites_a_line_number`, `no_comment_says_the_same_thing_twice`,
   `a_pub_fn_nobody_calls_is_dead`.
8. **A ceiling row that stops being needed is retired, never raised.**
9. Every widened item carries the "pub for the tests' sake" note.
10. A file that genuinely cannot move is on the exception table, with a reason.

## 3. The mover — how to use it, and what it already handles

```
python3 scripts/move-inline-tests.py src/core/executor.rs tests/core/executor core::executor
python3 scripts/move-inline-tests_test.py      # the mover's own test: 10/10
```

It handles, and the fixture proves each: a flat module; several modules (the
grouping is kept); an inner `#[cfg(unix)] { }` block in a helper; a `format!`
argument that runs over two lines; `use super::{A,B}` and `use super::sibling::X`
in one module; a cfg gate above an import; a `super::Item` path rather than an
import; an `include_str!` path relative to the src file; a `#[cfg(test)]`
constructor that stays inline. It never decides an owner and never weakens an
assertion.

**Environment for every cargo command on this host** — the system cargo has no
`fmt` subcommand, and the rustup toolchain's `rustc` needs libz:

```bash
export LD_LIBRARY_PATH="/nix/store/lw86vj9kfwgvlivg3p8qg2nibpmhmsl9-zlib-1.3.2/lib"
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export RUSTFMT=/nix/store/k2hwcha2p1fy0v0py34vq3jyvwqk339p-rustfmt-1.97.1/bin/rustfmt
# and for the Windows cross-target:
export PATH="/tmp/mingw-cc/bin:$PATH"
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-cc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-c++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
RUSTFLAGS="-D warnings" cargo check --target x86_64-pc-windows-gnu --lib --tests
```

## 4. The next thing: `src/core/executor.rs`

It is the only file still over the 3,000-line ceiling (3,053), and it holds
**42 tests in four modules** (`path_lookup_tests`, `child_process_tests`,
`search_read_tests`, `exit_status_tests`) plus three `#[cfg(test)]` helper
constructors.

The move itself is *verified* — it has been run and produced a compiling tree. The
two things left are known and small:

1. **Visibility.** The mirror reached these items, all in `src/core/executor.rs`:
   `with_idle` (a `#[cfg(test)]` constructor that must **ship**, because it mints no
   authority — this is *not* the C3 case), `fabricate_status`, `open_lock_at`,
   `ensure_status`, `suppress_pagers`, `detail_for_user` (all `pub(crate)`), the
   fields `reader`, `stdin`, `idle`, and `SudoKeepalive`'s **positional** field — and
   `has_bytes` in `src/core/launch.rs` (also `pub(crate)`). One widening script for
   the whole file was drafted and lost; write it again, apply it, and the build is
   clean.
2. **The ceiling row.** With the module gone the file is ~1,946 lines, so the
   `TOO_BIG_FOR_NOW` row for `src/core/executor.rs` is **retired, not raised** — the
   gate checks both directions.

Then `cargo fmt` and the chain. **Do not run `cargo fix --allow-dirty` on this
mirror**: it removes imports that are used only under `#[cfg(windows)]`, and the
Windows cross-target then fails. Fix the unused-import warnings by hand, per
platform.

After `executor.rs` the same shape repeats for the 29 files in the 1,000–1,999
band, then 58 at 400–999, then 85 under 400 — **179 of 236 src files still carry an
inline test module.**

## 5. What is deliberately not built

- **The 159 flat files' assignment** — each needs an owner (e6e / review / mirror).
  The classification is mechanical (what the test drives), so a script plus a
  table, not 159 judgement calls.
- **The exception table** for files that need `cfg(test)` inline — the gate that
  audits it in both directions is not written.
- **#117's remaining five subjects** — `prune_snapshots_after_sync`,
  `execute_transaction`'s counters, `heal` (8), `rebuild_kernel_modules` (4), and
  `sync`'s `PurgeUndeclared` arm. All need the machine fixture that
  `tests/a_machine_converges_tests.rs` already builds.
- **#123** — the registry-budget test's load-sensitivity, whose honest fix is a
  measurement in a fresh process.

## 6. One thing the next session should not redo

`scripts/widen-for-tests.py` is an unfinished experiment at making the widening
loop automatic. **It works for named items; the tuple-struct field and the
cross-file search loops need re-testing.** The reliable shape is a small script
that lists this one file's exact edits, which is what section 4 describes.

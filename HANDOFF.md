# HANDOFF — the #120 test-layout sweep, as of 2026-10-23

**Start here.** The next session does the same thing this session did, one file at a
time, now that both halves of the tool are in the repo. The state is green and pushed.

---

## 1. Where the tree stands

`main` is green and pushed. The verify chain passes on the committed tree:

| step | result |
|---|---|
| `cargo build --all-targets` | clean |
| `cargo test --no-fail-fast` | 1,932 lib + 1,072 suite, none failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | clean |
| `cargo fmt -- --check` (rustfmt **1.97.1**) | clean |
| `cargo check --target x86_64-pc-windows-gnu --lib --tests` | clean |

**`TOO_BIG_FOR_NOW` is empty, and that is the target state, not a regression.**
`src/core/executor.rs` was its last row; it is now 1,962 lines with 42 tests in
`tests/core/executor/`, and the row was retired rather than raised. The next file to
cross 3,000 lines without a row is a subject the split has not reached yet.

**Closed:** #116/#118/#119/#121/#122. **Open:** #120 (this sweep), #117 (six of
eleven sync-engine subjects; the rest need the machine fixture), #123.

## 2. The acceptance criteria

A batch is done only when **all ten** hold. They are the mover's own test —
`python3 scripts/move-inline-tests_test.py` — so a broken tool fails before a broken
tree does.

1. `cargo build --all-targets` clean.
2. `cargo test --no-fail-fast` none failed.
3. **No test lost** — the lib target falls by exactly the tests that moved and the
   suite rises by the same number. `cargo test --lib` before and after, compared.
4. Clippy `-D warnings --all-features` clean.
5. `cargo fmt -- --check` clean (rustfmt 1.97.1, what CI pins).
6. `every_test_file_is_in_the_suite` passes.
7. The scanning gates pass (`a_module_is_a_subject_not_a_pile`,
   `no_comment_cites_a_line_number`, `no_comment_says_the_same_thing_twice`,
   `a_pub_fn_nobody_calls_is_dead`).
8. A ceiling row that stops being needed is retired, never raised.
9. Every widened item carries the "pub for the tests' sake" note.
10. A file that genuinely cannot move is on the exception table, with a reason.

## 3. The pipeline — four commands per file

```bash
# environment (the system cargo has no `fmt`; the rustup rustc needs libz)
export LD_LIBRARY_PATH="/nix/store/lw86vj9kfwgvlivg3p8qg2nibpmhmsl9-zlib-1.3.2/lib"
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export RUSTFMT=/nix/store/k2hwcha2p1fy0v0py34vq3jyvwqk339p-rustfmt-1.97.1/bin/rustfmt
# (the Windows cross-check additionally needs /tmp/mingw-cc/bin and the three CC_ vars)
```

1. **Move.** `python3 scripts/move-inline-tests.py SRC DEST core::path`
2. **Widen.** A small per-file script listing the exact items the mirror reaches.
   Write it by hand from the compiler's `E0603`/`E0616`/`E0624` errors — the
   `executor.rs` one is at `/tmp/opencode/widen-executor.py` as a template.
3. **Fix the imports.** `python3 scripts/fix-mirror-imports.py --apply` — asks both
   compilers which imports are dead, and gates or drops each *name*.
4. **Verify.** `cargo fmt` → build → clippy → `cargo test --lib` (count) → Windows
   `cargo check` → retire any ceiling row → commit.

**Order matters and cost hours to learn:** widen *before* fixing imports, because a
successful widen changes what the compiler reports (an import that "could not be
resolved" is not yet "unused"). And never run `cargo fix` over a mirror: it deletes
imports only the other platform can see.

## 4. What is left

- **179 src files with an inline test module.** In size order:
  `src/parsers/language.rs` 1,155, `src/parsers/windows.rs` 1,128,
  `src/verbs/plan.rs` 1,942, `src/core/exit_policy.rs` 1,579,
  `src/utils/file.rs` 1,227, `src/parsers/ecosystem.rs` 1,188, `src/core/argv.rs`
  978 — then 29 in the 1,000–1,999 band, 58 at 400–999, 85 under 400.
- **159 flat files in `tests/` with no owner** — each needs assigning (e6e / review /
  mirror). Mechanical, so a script plus a table.
- **The exception table** for files that must keep `cfg(test)` inline, and the gate
  that audits it in both directions.
- **#117's five remaining subjects** and **#123** (below).

## 5. Deliberately not built, and why

- **`scripts/widen-for-tests.py`** (untracked) is a failed experiment at automating
  the widening. **Do not use it.** It mangled five tuple structs and wrote `pub`
  into a struct initializer. The per-file script is the reliable shape.
- **The mover still carries the src file's imports wholesale**, which is why step 3
  exists. Folding that into the mover is possible but needs both compilers, which
  belongs to the fixer, not the mover.
- **#117** — `prune_snapshots_after_sync`, `execute_transaction`'s counters, `heal`,
  `rebuild_kernel_modules`, and `sync`'s `PurgeUndeclared` arm. All need the machine
  fixture `tests/a_machine_converges_tests.rs` already builds.
- **#123** — the registry-budget test's load-sensitivity; fix by measuring in a fresh
  process.

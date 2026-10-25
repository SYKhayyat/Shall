# HANDOFF — the #120 test-layout sweep

**Written 2026-10-24. Read this first; it is the whole operating manual.**

The objective is one thing: **no `src/` file keeps its own tests.** Every
`#[cfg(test)] mod` moves to a mirror folder in `tests/`, one file per assertion,
until the library builds without `cfg(test)` and every test the library had still
runs. Ceilings are retired, not raised. Nothing is ever deleted to make the move
easier.

---

## 1. Where the sweep is, in numbers

| | |
|---|---|
| src files still carrying an inline test module | **85**, split: app 37, core 1, backends 32, model 0, parsers 0, verbs 7, config 4, utils 4 |
| tests already in mirror folders | executor, guard, transaction, generic, statement, sync, scheduler, language, windows, ecosystem, dnf, common, pacman, main, plan, resolve, vars, service, file, extras_lock, grammar, model, parsers, core |
| `TOO_BIG_FOR_NOW` rows | **0** — the target state, and `executor.rs` was the last |
| moved in this session | **1,515 tests** from 82 files; model and parsers now have no inline test modules, and core is down to 1 file |
| converter scripts in the repo | 2, both with passing self-tests |

`cargo build --all-targets`, clippy at `-D warnings --all-features`, and `cargo fmt -- --check` are
green after the current batch. The focused core mirror reports **440 passed, 0 failed**. The latest
focused app, backend, and config mirrors each report **26 passed, 0 failed**. The newest focused
mirrors report **26 link, 25 planner, and 18 config tests**, all passing. The library target
reports **693 passed, 0 failed**. The latest setting and NixOS mirrors each report **24 passed,
0 failed**. The latest full test run reached the suite and reported **2,338 passed, 1 failed**
on this host: the existing `argv_drift_tests::every_subcommand_shall_invokes_still_exists_upstream`
gate cannot read help from sandbox PATH wrappers for installed `apk`, `brew`, `bun`, `composer`,
`conda`, `dnf`, `gem`, `go`, `pacman`, and `pipx`. That is an environment/tool-discovery
failure, not a moved-test failure. The integration suite now runs **2,339 tests** including the
newly moved app/backend/config tests.
The suite wiring audit and both converter self-tests pass. The Windows cross-check is still unavailable
here because the cross compiler is not installed.

**The next moves, in this order.** Priority is inline-test count, not file size —
a file with 26 tests is 26 hand-edits if the tool does not work, and it will not.

1. Continue through the remaining app/backend files, then verbs/config/utils and the core long tail.

---

## 2. The pipeline — four commands per file

```bash
# 1. Move
python3 scripts/move-inline-tests.py src/core/foo.rs tests/core/foo core::foo

# 2. Widen — a small per-file script you write by hand from the compiler's errors.
#    (no automated widen script; see §6)

# 3. Fix the imports — ask both compilers, gate or drop each name
python3 scripts/fix-mirror-imports.py --apply

# 4. Verify
cargo fmt && cargo build --all-targets && cargo test --lib \
  && cargo clippy --all-targets --all-features --locked -- -D warnings \
  && cargo test --no-fail-fast
```

**Read an existing mirror before running anything.** `tests/core/executor/mod.rs` is
the smallest one and its shape is the one to copy: a `//!` line naming the mirrored
item, then one `mod` per inline test module, then the shared helpers, then the imports.
`tests/app/sync/guard/mod.rs` is the largest and shows the module-doc-comment style
the repo prefers for a mirror that earned its place.

---

## 3. The environment — this host, not a generic one

**Every cargo command needs these exports, in the same shell invocation.** The system
`cargo` at `/usr/bin/cargo` has no `fmt` subcommand and its `rustc` needs libz, so a
command without them fails in a way that looks like a code error:

```bash
export LD_LIBRARY_PATH="/nix/store/lw86vj9kfwgvlivg3p8qg2nibpmhmsl9-zlib-1.3.2/lib"
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export RUSTFMT=/nix/store/k2hwcha2p1fy0v0py34vq3jyvwqk339p-rustfmt-1.97.1/bin/rustfmt
```

**`RUSTFMT` must be 1.97.1, not whatever is default** — that is the version
`.github/workflows/ci.yml` pins, and a locally-clean run at a different version is
the trap this repo already documented once.

**`git commit` needs the same exports in the same command.** The pre-commit hook runs
`cargo fmt -- --check`, and without `PATH` carrying the toolchain it refuses a commit
that is correctly formatted. If a commit is refused with *"Run 'cargo fmt'"*, check
your exports before believing it.

**The Windows cross-check is the fifth step of the chain and it is not optional** —
CI runs on Windows, and `scripts/unix-check.sh` needs Docker (absent here). From this
host:

```bash
export PATH="$(dirname $(command -v x86_64-w64-mingw32-cc 2>/dev/null)):$PATH"
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-cc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-c++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
RUSTFLAGS="-D warnings" cargo check --target x86_64-pc-windows-gnu --lib --tests
```

`-D warnings` matters here: two of the fourteen historical Windows errors were
`unused_variables`, fatal only under CI's gate.

**Never run anything over ~30 seconds in the foreground.** The suite is ~2 minutes,
a first Windows cross-check ~25 (all 448 target crates, then ~40s), a full
`cargo mutants` longer than everything. Background it and do the next thing.

---

## 4. Rules the repo enforces, that a layout change can break

Each of these is a test that fails loudly. Read the test before fighting it.

- **`a_module_is_a_subject_not_a_pile_tests`** — a file over the ceiling needs a row
  in `TOO_BIG_FOR_NOW`, **and the table is audited in both directions**: a file that
  drops back under the line must have its row deleted. `src/core/executor.rs` came off
  this way at the end of 2026-10-23. Do not add a row to make a gate pass.
- **`every_test_file_is_in_the_suite`** — `tests/` has `autotests = false`; a file is
  not a `mod` somewhere, it does not run. Every new mirror file and folder needs
  wiring.
- **Comments (P6):** a comment states a constraint the code cannot show. Not what a
  line does, not `V.n` citations as design documents. Prefer a name or a type.
- **`no_comment_cites_a_line_number`** — do not write "line 1318"; write the name.
- **`a_pub_fn_nobody_calls_is_dead`** — widening something only tests call is fine,
  but it must carry the note in §5 or it reads as dead code.
- **Widen, don't duplicate.** If a test cannot see an item, widen that item. Never add
  a second implementation or a `#[cfg(test)]` constructor that ships.

---

## 5. The widening, and the two shapes it has

A test crate links the library *without* `cfg(test)`, so a private item is invisible
there. When the compiler says `is private` (`E0603`/`E0616`/`E0624`), widen it and
put a note on it:

```rust
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub reader: Arc<dyn ExecutionLayer>,
```

**There are two shapes, and telling them apart is the C6 decision:**

1. **The tests' sake** — as above. This is most of them.
2. **It ships.** A `#[cfg(test)]` helper that mints no authority — it only sets fields
   a constructor already has, or is a plain value holder — should be published and
   un-gated, *not* stuck behind `pub(crate)` so a test can call it.
   `RawExecutor::with_idle` was exactly this and is the example to copy (it says so in
   its doc comment). C3 stays the ruling for anything that *does* mint authority:
   that one stays inline.

**Write a small script per file listing the edits, and keep it out of the repo.** For
`executor.rs` the record was `/tmp/opencode/widen-executor.py`. Do not try to
automate this — see §6.

---

## 6. What not to redo, and why

These cost the most time. All are written down rather than re-derived.

- **Do not automate the widening.** `scripts/widen-for-tests.py` is an abandoned
  experiment, left untracked and deliberately not deleted only as a warning. It
  mangled five tuple structs into invalid syntax and wrote `pub` into struct
  initializers. The compiler's errors are a short, precise list; a hand script that
  carries them is faster than an automation that has to guess where an item's body
  ends.
- **Widen before fixing imports, not after.** A successful widen changes what the
  compiler reports: an import that *could not be resolved* is not yet *unused*. Run
  `fix-mirror-imports.py` after the widening and the build is clean, and running it
  before leaves unusable output. This ordering is not obvious and cost a full cycle.
- **Never `cargo fix` a mirror.** `--all-targets` is single-platform: it deletes
  imports that are used only inside `#[cfg(windows)]` blocks, and the Windows cross
  check then fails with `E0425`/`E0433` for names that are gone on Linux and alive on
  Windows. That is exactly the bug `fix-mirror-imports.py` exists to avoid, by asking
  both compilers and classifying each **name** rather than each line.
- **A name, not a line.** `use shall::core::{Error, ExitPolicy}` where one platform
  calls both dead and the other needs one of them: the statement is split
  `#[cfg(unix)]`/`#[cfg(windows)]`, not deleted.
- **Run the tools' own tests after any change to them.** `scripts/move-inline-tests_test.py`
  (11/11) and `scripts/fix-mirror-imports_test.py` (12/12). Both run in about a second
  against fixtures and no compiler. **Ten full cargo builds were spent discovering
  what a fixture test finds immediately** — this is the one lesson from the retro that
  is binding on the next file.
- **Verify the count, not the colour.** For a move to be done, `cargo test --lib` must
  fall by exactly the number of tests that moved and the suite must rise by the same
  number. A green suite over a silently dropped test is the failure this catches.

---

## 7. Deliverables and definitions of done

Per file: all ten criteria hold.

1. `cargo build --all-targets` clean.
2. `cargo test --no-fail-fast` none failed.
3. **No test lost** — lib fell by N, suite rose by N, measured.
4. Clippy clean at `-D warnings --all-features`.
5. `cargo fmt -- --check` clean at 1.97.1.
6. `every_test_file_is_in_the_suite` passes.
7. The scanning gates pass (ceiling, comment rules, dead `pub`).
8. A ceiling row that stopped being needed is retired.
9. Every widened item carries its note.
10. A file that genuinely cannot move is on an exception table with a reason — the
    table does not exist yet, and it is a deliverable (§8).

But **the sweep is not one file at a time for 183 files.** Batch 3–5 small files per
verify cycle once `parsers/` has proved the path is repeatable: the wall-clock is the
cargo check, not the edit, and the two smaller parsers are near-certain to need
identical widening for the same parser-shaped items.

---

## 8. Still to build, outside the move itself

- **`tests/parsers/mod.rs` and the flat-file assignment.** 159 flat files in `tests/`
  have no owner yet. Each is classified by what it drives — parser fixture, review
  output, or a mirror of an item that no longer has a src home. Mechanical, so a
  script plus a table rather than 159 judgement calls.
- **The exception table** for files that must keep `cfg(test)` inline, and the gate
  that audits it in both directions.
- **Issue #117**, five remaining subjects: `prune_snapshots_after_sync`,
  `execute_transaction`'s counters, `heal`, `rebuild_kernel_modules`, and `sync`'s
  `PurgeUndeclared` arm. These need the machine fixture that
  `tests/a_machine_converges_tests.rs` already builds.
- **Issue #123**, the registry-budget test's load-sensitivity, whose honest fix is to
  measure in a fresh process.

Closed this week and not reopened: #116, #118, #119, #121, #122.

---

## 9. Repo and remote

```bash
git remote -v          # https://github.com/SYKhayyat/Shall.git
git log --oneline -6   # six leadership commits, each with the full reasoning inline
```

Read `docs/SPEC.md` first for the rules of the road, and `docs/spec/decisions.md`
for every open question — an ID in that register (`D*`, `W*`, `K*`, `N*`, `T*`, `U*`)
is the owner's ruling, not a coder's. **If Part II of a spec ever looks wrong, stop
and ask; do not fix it in passing.**

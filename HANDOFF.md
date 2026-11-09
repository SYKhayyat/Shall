# Handoff — 2026-11-08, second session (`#139`, `#140`, the CI reds)

**Branch `main`.** The first session's three filings were pushed (`10b72fa6`) at the start of
this one. This session opened with the scheduled nightly `42469891783` (3h37m, **9 of its jobs
red**) and worked outward from it.

| # | Subject | Commit |
|---|---|---|
| `#135`, `#136`, `#138` | pushed — built in the previous session, never pushed | `10b72fa6` |
| `#139` | one upstream 504 printed three red lines, and Shall called it `unknown` | this session |
| `#140` | the storage image's single-source, no-retry ZFS fetch | this session |
| `#142` | scoop's download exception was read as a wrong manifest | this session |
| `#141` | the mutation gate's timeout was derived from a solo baseline | this session |
| — | the test fixture read the developer's `~/.npmrc` and called the answer Shall's | this session |
| — | the `stack` row left `windows-ci`'s terminator floor, by the `conda` rule | this session |
| `#18` | the concurrency ceiling was taken from the loop that releases it | this session |
| `#21` | the release path is four files that must agree, and nothing checked | this session |

- **`#139` — a download's status is classified by its code.** `github:`, `web:` and `appimage:`
  folded every non-success status into `Error::Other` → `Retryability::Unknown` →
  `shall-failure-class: unknown`, which both harnesses read as *a defect*. `Error::HttpStatus`
  carries the status, `retryability()` answers from it (5xx transient, other non-success
  permanent, 429 keeps `RateLimit`'s own verdict), and the three backends build it through one
  constructor. `with_note` attaches to the detail, so the verdict survives the `while applying`
  line — the bug `with_note` was written to fix, one variant over. Ruling `C13`, rule **II.77**,
  reason **V.227**.
- **`#139`, the harness half.** A backend whose install failed for a reason Shall classed as
  passing was *also* shrinking the real-lifecycle ratchet and the section floor, so one 504
  reported both "a defect" and "coverage collapsed", and the two shadows named no cause of their
  own. `tally_unmeasurable` — one function, lifted by `harness-logic-test.sh` and called by both
  harnesses — now counts those backends toward the floor and names them, in the same bucket as a
  dated `drift` excuse. Only a backend nothing accounts for stays uncounted.
- **`#140` — the storage image's fetch has retries and two doors.** `Dockerfile.storage` fetched
  the OpenZFS tarball once, with no retry; a `504` at `github.com` meant the image was never
  built and the whole ZFS shard reported no lifecycle. `--retry 5 --retry-all-errors` on the
  release URL, then the same against `codeload.github.com`'s archive of the same tag (verified
  200 from here). No third-party mirror: an unverified archive is worse than a leg that is
  occasionally red.
- **`#142` — scoop's `catch` was read as a verdict on the manifest.** `lib/download.ps1` prints
  `URL <url> is not valid` inside the `catch` around its download call, whatever the exception
  was, so the sentence names a URL as the suspect when the transfer is what failed. The Windows
  sweep reds `scoop:jq` on it against a `jqlang/jq` asset that answers `200` from every machine
  asked since, and scoop's own manifest carries a hash that matches. Two markers added
  (`is not valid`, `download failed`), and the widening is bounded on both sides by tests: a
  clean `Couldn't find manifest for 'x'` stays permanent and absent, and when both arrive
  together transient wins — `retryability` consults it before `absent` on purpose.
- **`#141` — the mutation ceiling was derived as though a mutant ran alone.** Four shards exited
  3 with `TIMEOUT` and no survivor list. Both `guard.rs` mutants that timed out are caught
  locally inside the old ceiling (~210s of suite against 610s), so the number was not too small
  for the work: it was too small for two mutants at once, each running the whole suite with
  `--retries 2`, on a shared four-core runner. `--minimum-test-timeout 1800` bounds the derived
  number from below. A review test pins the flag, because the shard's whole value is reporting a
  survivor.
- **`stack` left `windows-ci`'s floor line, and only that line.** The runner still carries the
  toolchain — the probe drove it and it is not in `absent` — but no verb resolved the operand, so
  the run printed `inconclusive: stack` and the owed name could not be paid. The same nightly's
  ubuntu leg measured it (`divergent: stack … this one honours the terminator`) and keeps the
  name, as does `container-tools`. No row loses evidence: `stack`'s row is `Evidence::Divergent`
  and its refusal is what ships either way, so this leg was never the leg its claim rested on.
  This is the `conda` rule from `#134`, and the reasoning is in the file.
- **`retry.sh` waits long enough to matter.** Its 3 attempts were spaced 5s and 10s, and the
  nightly lost four attempts at `github:sharkdp/fd` and four more in the arch build to `504`s
  within ~25 seconds — the backoff never crossed the far end's own recovery. Now 15s, doubling.
- **`#18` — a permit taken on the coordinator's loop is a permit taken from the thing that
  releases it.** With a graph wider than `max_concurrent` the arrangement was self-blocking: the
  loop awaited permits held by tasks it could not reach `join_next` for, and the loop is what
  records the completions that free them. The permit is taken inside the spawned task now; the
  limit is unchanged and the loop records a finished batch when it finishes. A closed semaphore
  reports every member of the batch as failed through the skipped-node shape, so a shutdown is
  counted rather than looking like lost work. **Restored to the near side the new test does not
  go red — it deadlocks**, which was measured (killed at 120s) and is recorded in the test's own
  comment. Ruling `C14`, rule **II.78**, reason **V.229**.
- **`#21`, the release half — four files decide what a user downloads and none can see the
  others.** The build matrix, the release job's asset naming, `install.sh` and `install.ps1`. Each
  is reasonable alone; the only run that would show a disagreement is the tag push. Three tests
  now parse each file in its own shape and compare the sets in every direction: nothing built
  that is not published, nothing published that is not built, every installer asking for a
  published target, every published target reachable from a documented line (the two `-linux-gnu`
  rows exempt, because the installer's header publishes them as opt-in). Both directions watched
  red — dropping a target from the release list failed two tests, a typo in `install.sh`'s triple
  failed the third.
- **`#18`, sub-task 4 — the preview's phase order was correct and nothing asserted it.** The engine
  asks removals first and the install ceiling after; `preview_refusals` did the same and no test
  said so. Three tests now pin it: the removal refusal comes before the install one, each tripped
  ceiling names its own setting, and a run inside every ceiling says nothing. Putting the install
  question first was watched failing the order test. The order decides the *count* as well as the
  name, because every phase spends against one `Reaping` budget.
- **`#21`, the local gate.** `release-check.sh` ran neither `decision-count.sh --check` nor the
  release-metadata check, so a local GO could ship a tree whose `SPEC.md` and `decisions.md`
  disagreed. Both are gates in it now.
- **The fixture bug the red suite found.** `grade2_wedged_manifest_tests` passed on CI and failed
  here, and the reason was not the product: the harness pointed `HOME` at the fixture but npm
  reads its global prefix from `~/.npmrc`, so the fixture's `npm install -g` was writing to the
  **developer's** `~/.npm-global`. `TestKernel` now sets `NPM_CONFIG_USERCONFIG` and
  `NPM_CONFIG_PREFIX` too — the sixth setting to arrive for the sixth time this way.
- **Verified on this tree, after the last edit:** `cargo build --all-targets`, `cargo test
  --no-fail-fast` — **3100 passed, 0 failed** in 206.45s; `cargo clippy --all-targets
  --all-features --locked -- -D warnings`; `cargo fmt -- --check`;
  `scripts/decision-count.sh --check` (251 entries, 246 ANSWERED, 0 OPEN);
  `scripts/harness-logic-test.sh` — **103/103**; `shellcheck -S warning` on both harnesses,
  `retry.sh` and the two touched scripts.
- **Three hand mutations watched red:** `(500..600).contains(status) → false` failed 2 of the 5
  new classification tests in 0.03s; `unrecorded) EXHAUSTED_UNREGISTERED=…` deleted from the
  Windows tally failed `harness-logic-test.sh` by name; and removing scoop's two new markers
  failed **3 of 4** cases in `a_scoop_url_that_did_not_download_is_not_a_wrong_manifest`.
- **Not verified:** no container leg and no Windows run (neither is available on this host); the
  `#139`/`#140` evidence will be the next nightly — a `github:` install that retries and clears
  where it used to red the leg, and a storage build that succeeds.

# Handoff — 2026-11-08 (`#135`, `#136`, `#138`)

**Branch `main`.** Three filed issues built, each with its ruling in the register, its rule in
Part II and its reason in Part V.

| # | Subject | Commit |
|---|---|---|
| `#129`, `#109`, `#25` | three entries said open or unbuilt when the evidence said otherwise | `a67c35a6` |
| `#117` | the sync engine's last 12 mutation survivors, and the transaction shard's last one | `5f70c6d1` |
| `#135` | xbps's listing read a database row as an installation | this session's commit |
| `#136` | a removal that leaves the launcher behind says so, and deletes nothing | this session's commit |
| `#138` | the data-lock shard's timeouts were the process-wide `held` count | this session's commit |

- **`#138` — the self-hold question is per directory.** `held()` was a process-wide counter, so
  `for_this_write` skipped the lock whenever the process held *any* lock — a write into another
  data directory raced, and three mutants of the count slept out `WAIT_SECS` in every e2e leg and
  timed the mutation shard out at 825s. `held_on(path)` reads a set of canonical paths, the
  handle keeps the key it recorded so `Drop` removes exactly that entry, and `for_this_write`
  asks about the directory it writes under. Its acquire refuses at once when the owner stamp is
  this pid; a direct `acquire` still waits for a same-process holder (`watch` depends on it).
  Ruling `C12`, rule **II.76**, reason **V.226**. The module's 21 tests run in ~4s.
- **`#135` — the lister was the bug.** `xbps-query -l` opens every row with a state word, and the
  reader took the first token that parsed as a pkgver. `parse_xbps_list` now reads it
  (`XBPS_STATE_WORDS`, exhaustive), a row dropped for its state is an answer rather than an
  unrecognised shape, and `-m`'s bare pkgver still counts. The harness witness was its sibling:
  `canary_manager_state` asked `xbps-query <pkg>` (the repository index) and now reads `-l` and
  prints the state word itself. Ruling `C10`, rule **II.74**, reason **V.224**.
- **`#136` — the sentence is in the engine, because every removal passes through it.**
  `leftover_entries` decides by name (exactly the package or a dotted extension — `pv` never
  accuses `pvx.cmd`), `leftover_warning` names the path and whether the directory is on PATH, and
  `SyncEngine::sync` says it after a successful, non-reverted run that carried past no failure.
  Both harnesses now treat their `removal_leaves_binary` entry as the expected leftover: they
  pass only when the removal's own output named the file. Ruling `C11`, rule **II.75**, reason
  **V.225**.
- **Verified on this tree, after the last edit:** `cargo build --all-targets` (through the test
  build), `cargo test --no-fail-fast` — **3091 passed, 0 failed** in 323.51s; `cargo clippy
  --all-targets --all-features --locked -- -D warnings`; `cargo fmt -- --check`;
  `scripts/decision-count.sh --check` (250 entries, 245 ANSWERED, 0 OPEN);
  `scripts/harness-logic-test.sh` — **101/101**; `shellcheck -S warning` on the three touched
  scripts.
- **Two hand mutations watched red for `#138`:** `held_on` restored to the old process-wide flag
  failed 11 tests in 1.7s, and `held_on -> false` failed in 0.02s — the re-entrant door's stamp
  check is what keeps a later `for_this_write` from sleeping out the wait, and it answers from
  the stamp rather than from the function the mutant replaced.
- **Not verified:** no container leg (the harness change is shell-only and its predicates are
  pinned by `harness-logic-test.sh`, but no real manager was driven); `x86_64-pc-windows-gnu` not
  re-checked — the target std and mingw toolchain are not installed on this host, and nothing in
  the change is `cfg`-gated; `scripts/unix-check.sh` not run because this host compiled the unix
  half natively.

### The second nightly (`42523470970`), and what it found

Dispatched to confirm the first batch. It confirmed `#139`, `#140`, `#142` and `#141` — the
`github:`/`web:`/`appimage:` statuses, the storage fetch and the mutation ceiling were all
silent — and then found five more things, four of which were **mine**:

- **`#19`, an unclosed block refused at line 0** — which is not a line in any file. Both parsers
  carried a bare `nested: bool` instead of the header's origin. And the gated reader's arm, which
  the audit assumed unreachable, did not fire at all: a truncated `priority` file parsed clean and
  kept its conditions as global ones. Ruling `C15`, rule II.79, reason V.230.
- **Two tests that only ever ran on Linux.** `appimage`'s `detect` is `cfg!(target_os = "linux")`,
  so the leftover-launcher test skipped the backend on macOS and Windows and read the skip message
  as a missing sentence. `github` deploys to the same directory and exists everywhere. Reason V.231.
- **A harness scoring a fix as a failure.** `assert_binary_gone`'s caller in the Windows sweep used
  `gone_ok` for a package the run installed, so nimble's correctly-named leftover read as
  "still there". Now routed through `assert_binary_gone`.
- **Two mutation survivors** in `datalock.rs` that `#138`'s work left: `holder_is_this_process` →
  `true`, and a `&&` → `||` that was measured **unreachable** (recorded in the test rather than
  faked). The reachable one is caught.
- **A survivor in the sync engine**, `!reverted && matches!(result, Ok([]))` → `||`: the leftover
  sentence fired after a revert and after a carried-past failure. Extracted to
  `owes_a_leftover_sentence` with five tests; the exact mutant is now caught by four.
- **opensuse's `zypper install` had no retry.** One mirror answered 404 for three packages in one
  `refresh`, exit 4, whole leg red before a test ran. Wrapped in `retry.sh`, and `retry.sh`'s own
  header — which said distro package layers are deliberately left to fail loudly — now records the
  measured exception rather than being quietly contradicted.

### Where the issue list stands

Closed this session: `#135`, `#136`, `#138` (built previously, pushed at the start of this
session), `#139`, `#140`, `#141`, `#142`, and `#17` (audited open — six of its seven candidates
were already closed and the seventh's proposed fix would break its only caller). Commented with
what is done and what remains: `#18` (sub-tasks 1 and 4 done, 2/3/5/6/7 open), `#19` (five of
eight closed, four open, next step named for each), `#20` (TUI gap closed, stale-session gap
needs a ruling), `#21` (release/install half done, three items open).

Still untouched and owed a conversation rather than a commit: `#94` and `#124` (owner decisions),
`#59`–`#67` (the `adopt`/`export` feature family), and `#39`–`#44` (the Lamdan/interop prose
audits, which delete large spec prose).

### What to take next

1. **The next nightly** answers all three: `#135`'s crash section check (5) on `void`/`tools`
   (the witness now prints the manager's state word and the sync's rc), `#136`'s Windows sweep
   (now asserting the removal named the file rather than excusing it), and `#138`'s datalock
   shard (which should reach its survivor list instead of timing out).
2. **`#100`** — the next `void` leg names uv/yarn. Needs a run.
3. **`#130`'s unproven half** — a real `schtasks`/`launchd` provisioning after a `schedule` edit.
4. **A container leg since `C9`** — nothing in the matrix has run against the widened reads, and
   the removal guard's essential-list query is the one to watch.

---

## Where the tree was — 2026-11-03 (`#111`)

State of the tree, what is verified, what is not, and how to continue. Read `CLAUDE.md` and
`docs/SPEC.md` first; they still govern everything below.

## Where the tree is now (2026-11-03)

**Branch `main`, clean, pushed.** One issue this session, ruled and built:

| # | Subject | Commit |
|---|---|---|
| `#111` | a fan-out over managers Shall only *waits* on was bounded by the core count | this session's commit |

- **`#111` is ruled `C9` and built.** Rule **II.73**, reason **V.223**, register entry `C9`. A
  third knob — **`spawn_parallel`, default 16** — bounds every fan-out whose children only *ask* a
  manager for an answer (a listing, a probe, a version); `max_parallel` keeps its core-count
  default and bounds the children Shall *tells* to change something. The axis is what the child
  does, so `sync`'s own reads moved with it and its writes did not. `generate:` stays on
  `max_parallel`: a user's script is not a manager.
- **`managed_pkgs` fans out one future per manager**, grouped by `app::stale_lock::lock_key` — so
  the budget is not spent on siblings waiting for the same listing, and `pacman`/`yay`/`paru`
  (three programs over one database) are asked one at a time. That grouping is `#111`'s own stated
  hazard, answered rather than assumed.
- **The instrument can say the degree now.** `report_if_over` is handed the budget, `slot_reading`
  is pure, and a wide run reports *"this run had 16 slot(s) for child processes and used 3 of
  them, over 17 child command(s)"* — the reading `#107` had to give up when it moved to width. A
  one-wide run is still the collapse verdict's, so one defect is still one sentence.
- **Measured, mock managers, 120 ms each, seven of them:** two slots → **490 ms, width 2, four
  waves**; sixteen → **122 ms, width 7, one wave**. On the CI runners' own table that is
  `sbom`/`export`/`audit` going from ~4.25 s to ~their slowest child (~2.0 s). Route 3 — measure
  and close as not worth doing — was available and the numbers did not support it.
- **Verified on this tree, after the last edit:** `cargo build --all-targets`, `cargo test
  --no-fail-fast` (3078 passed, **1 failed — pre-existing, see below**), `cargo clippy
  --all-targets --all-features --locked -- -D warnings`, `cargo fmt -- --check`, and
  `scripts/decision-count.sh --check` (0 OPEN, 242 ANSWERED, 247 entries).
- **Not verified:** no container leg re-run since the widening; the new budget has been driven
  through mocks, not through a real manager beyond this host's two; `x86_64-pc-windows-gnu` was
  not re-checked this session (nothing in the change is `cfg`-gated).
- **The one red on this host is not this change.** `grade2_wedged_manifest_tests::
  a_name_no_backend_can_install_is_never_left_in_the_manifest` fails **identically on the
  unmodified tree** (stashed and re-run: 3066 filtered, same panic). The cause is the host's `npm`
  being a read-only nix-store install: Shall's install runs `npm install -g`, whose global prefix
  resolves to `/nix/store/…-nodejs-slim-22.23.4/lib`, so npm dies with `ENOENT … mkdir` before it
  ever reaches the registry (`npm error 404 'shall-no-such-pkg-zzz-9@*' is not in this registry`
  is what a plain `npm install` says). Shall then reports *"Nothing classified the failure above"*
  and keeps the line — correct for an unclassified failure, and a `#54`-class clean-host
  assumption rather than a wedged-manifest defect.

### Environment, as it is on this host

Everything in the 2026-10-31 section below still holds, with three corrections:

- **The rust toolchain needs `LD_LIBRARY_PATH`** pointing at a zlib in the store, and cargo strips
  it for its children — so `RUSTC` (and `RUSTDOC`) must be a one-line wrapper that exports it.
- **`rustfmt` no longer needs a shim: use the rustup one** (`$HOME/.rustup/toolchains/stable-*/bin`,
  `rustfmt 1.9.0-stable`). The store path the earlier handoff names
  (`…-rustfmt-1.97.1/bin/rustfmt`) is **gone** — garbage-collected — and a `RUSTFMT` pointing at
  it makes `cargo fmt` fail with *"Could not run rustfmt"* while otherwise looking healthy.
- **A Windows link** needs mingw's gcc wrapper (in the store), plus `libpthread.a` and
  `libmcfgthread.a` via `-L` from a linker shim: the wrapper swallows `LIBRARY_PATH` and
  `NIX_LDFLAGS`. Recipe in `plan.md` Tier 37.

### What to take next

1. **`#112`** — dispatch a native sweep. `C7`, `C8` and `C9` are all in, so the run answers the
   lister question the issue was filed for, and `C9` gives the sweeps a wider fan-out at the same
   time. No harness change.
2. **`#100`** — the next `void` leg names uv/yarn. Needs a run.
3. **`#130`'s unproven half** — a real `schtasks`/`launchd` provisioning after a `schedule` edit.
4. **A container leg since `C9`** — nothing in the matrix has run against the widened reads, and
   the removal guard's essential-list query is the one to watch (it is now up to 16 wide on a
   path that ends in a deletion).

---

## Where the tree was — 2026-11-02 (`#133`, `#130`)


State of the tree, what is verified, what is not, and how to continue. Read `CLAUDE.md` and
`docs/SPEC.md` first; they still govern everything below.

## Where the tree is now (2026-11-02)

**Branch `main`, clean, pushed.** Two issues closed this session, each with a rule in the register
and a test that was watched failing first:

| # | Subject | Commit |
|---|---|---|
| `#133` | the Windows unconfined launch (`cmd /c start`, no `/wait`) returned before its child | `05ff7fa8` |
| — | the decision register said 244 where it held 245 (drift left by `#132`) | `466bc4d2` |
| — | `#133` measured under Wine 11.0, not just type-checked | `9d4af64d` |
| `#130` | `schedule add`/`remove` converged the whole machine to edit one line | this session's last commit |

- **`#133` is fixed and measured, not closed.** The flag is `/wait` and the argv is now data
  (`Sandbox::windows_unconfined_argv`). A Windows-target build of a scratch crate linking
  `libshall` was run under Wine 11.0: the shipped `Sandbox::wrap` on `Confinement::None` returns
  `Some(7)` for a child that exits 7, where the pre-fix line returned `Some(0)`. Wine's `cmd` is a
  reimplementation — the issue still closes on a host without Windows Sandbox, or on the stub
  `cmd` it proposes. Recipe (mingw `-L` shim, `wine64`) is in `plan.md` Tier 37 and the issue.
- **`#130` is ruled `C8` and built.** Rule **II.72**, reason **V.222**, register entry `C8`.
  `handle_schedule` no longer ends in `handle_sync`; it calls the same `apply_one` /
  `undo_statement` pair `apply` uses, so one phase table serves every one-line command. Tests in
  `tests/e2e/a_schedule_edit_converges_only_its_own_line_tests.rs`.
- **The harness now pins `XDG_CONFIG_HOME`.** Landing `#130` found that a real provisioning test
  had written and *enabled* `shall-tidy.{service,timer}` in the developer's own
  `~/.config/systemd/user`: `dirs::config_dir()` reads `XDG_CONFIG_HOME` before `HOME`, and the
  fixture only overrode the latter. The units were removed by hand; `tests/harness/mod.rs` sets
  the fifth variable, and `run_with_path` lets a test stand a program in (`systemctl`).
- **Verified on this tree, after the last edit:** `cargo build --all-targets`, `cargo test
  --no-fail-fast` (3068 passed, 0 failed), `cargo clippy --all-targets --all-features --locked --
  -D warnings`, `cargo fmt -- --check`, `cargo check --target x86_64-pc-windows-gnu --all-targets`
  with `-D warnings`, and `scripts/decision-count.sh --check` (0 OPEN, 241 ANSWERED, 246 entries).
- **Not verified:** the container matrix has not been re-run since `C7`; the macOS `launchd` and
  Windows Task Scheduler provisioners are not exercised by the `#130` tests (only the systemd arm,
  through a stub); `#112`'s native sweeps still need a dispatch.

### Environment, as it is on this host

Everything in the 2026-10-31 section below still holds, with three additions:

- **The rust toolchain needs `LD_LIBRARY_PATH`** pointing at a zlib in the store, and cargo strips
  it for its children — so `RUSTC` (and `RUSTDOC`) must be a one-line wrapper that exports it, and
  `cargo-clippy`/`cargo-fmt` need the same treatment on `PATH`. Without this, `cargo test` dies at
  `rustc -vV` with `libz.so.1: cannot open shared object file`.
- **A Windows link** needs mingw's gcc wrapper (in the store), plus `libpthread.a` and
  `libmcfgthread.a` via `-L` from a linker shim: the wrapper swallows `LIBRARY_PATH` and
  `NIX_LDFLAGS`. Recipe in `plan.md` Tier 37.
- **Wine is available**: `nix-shell -p wine64 --run 'wine …'`, `WINEPREFIX=/tmp/win133`.

### What to take next

1. **`#112`** — dispatch a native sweep. `C7` and `#133` are both in, so the run answers the
   lister question the issue was filed for, with no harness change.
2. **`#111`** — the manager fan-out is bounded by the core count and no instrument reports the
   degree it lost. No ruling needed.
3. **`#100`** — the next `void` leg names uv/yarn. Needs a run.
4. **`#130`'s unproven half** — a real `schtasks`/`launchd` provisioning after a `schedule` edit.
   The narrowed path calls the same phase applier those platforms already use, but only systemd
   has been driven.

---

## Where the tree was — 2026-10-31

- Branch `main`, HEAD `cf8eaafd`, pushed to `origin`. **`C7` and the harness family it dragged
  with it are in the working tree, uncommitted** — `git status` is the list: 40 tracked files
  across `src/`, `tests/`, `scripts/`, `docker/` and the spec, three new test files, and the
  `packages.rs` → `apply.rs` rename (`tests/verbs/packages/` → `tests/verbs/apply/` with it).
- **Verified on this tree, 2026-10-31/11-01**, all after the last edit: `cargo build
  --all-targets`, `cargo test --no-fail-fast` (3062 passed, 0 failed), `cargo clippy
  --all-targets --all-features --locked -- -D warnings`, `rustfmt --check` over every crate
  target, `shellcheck -S warning`, `cargo check --target x86_64-pc-windows-gnu --all-targets`
  with `-D warnings`, `scripts/harness-logic-test.sh` (79/79), `scripts/decision-count.sh
  --check`, and the container harness on **ubuntu (391/0/5), arch (402/0/6) and tools
  (470/0/8)**, all `PASS`. Those numbers and the two defects the run produced are in
  `docs/spec/plan.md`'s `C7` entry; the second one — the `SIGTTIN` stop on a manager that reads
  a terminal it does not own the foreground of — is **filed as `#132`, not fixed**: reproduced
  outside the container as `shall run -- sh -c 'read -r x; …'` never returning (child in state
  `T`, own `PGID`, somebody else's `TPGID`), so it is every terminal-handoff door — `shall run`,
  the ephemeral `shall shell`, `$EDITOR`, `self-upgrade`'s `cargo install` — and it has been
  broken since `83b06c11` (2026-08-24). The fix changes interactive behaviour, so it is the
  owner's call.
- What that leaves unverified is the same list as before: the native macOS/Windows sweeps (so
  `#112` itself still closes only on a dispatch), the container legs not run here (fedora,
  alpine, opensuse, void, gentoo, guix, slackware), and the behaviours only a real manager
  answers — the guard's OS-essential protection, crash/fault injection, real lifecycles outside
  the three images, and argv as installed.
- This session's commits, in order:
  - `aa970f12` — **#113** closed: `classify_install` names the real culprit; guix ratchet 3 → 4.
  - `cee823ae` — **#129** closed: bun's `(0 installed)` header is an empty machine, not an
    unreadable one.
  - `ac797219` — **#112**: `dir` named in the native sweep's refusal table; README follows the
    guix ratchet.
  - `5bf94cfc` — **#109**: the apt legs refresh the metadata the image baked, before the first
    install.
  - `cf8eaafd` — **`C7` (2026-10-27): the ruling lands** (register + II.70 + V.220). The code
    followed on 2026-10-30/31 as one verb — `apply` / `apply --delete`.

## `C7` — the work order it was built from

The owner ruled it after the #112 diagnosis (the comment on #112: *the dispatch run's evidence is
in*). The full work order — mechanism, call sites, acceptance criteria, and the tests — is the
third comment on #112, **`issuecomment-6475969648`**. In short:

`Reconcile` gains `named: Option<HashSet<String>>`; `PlanScope` gains `Named(keys)`
(`filter() → None`, `reaps() → None`, plus a `named() → Option<&HashSet<String>>`); the planner
filters the install side and the drift/absent loops to the named keys; narrowed runs skip repos,
the non-package phases and maintenance; `say_if_the_failure_was_not_yours` is deleted. Four call
sites: `handle_install`, `handle_uninstall`, `uninstall_as_absent`, `handle_teleport`.

**The tests exist and were watched failing in the previous session's design** — written, then
taken back out of the tree so `cf8eaafd` lands green. Copies on the author's box at
`/tmp/opencode/c7-tests/` and reproduced verbatim in the #112 comment. Re-create them, register
them in `tests/app/sync/planner/mod.rs` (after `a_plan_reaps_only_what_it_was_asked_about_tests`)
and `tests/e2e/mod.rs`, **watch them fail against the unmodified code**, then implement.

**`docs/spec/plan.md` now carries the whole outline** — the numbered mechanism, the four call
sites, what to skip in a narrowed run, the deletions, the sibling left alone (#130) and the exit
condition — so the next session does not re-derive any of it. That entry, not the chat, is the
build list.

## Working tree

Forty tracked files modified, the `apply` rename, and three new test files (see *Where the tree
is*). Untracked and deliberately never committed: `scripts/__pycache__/` and
`scripts/widen-for-tests.py`.

## Environment — no boundaries, and how

The host is NixOS 26.05 (`desktop`), 8 cores, 11 GB. There is **no Docker**, but there is
**podman working rootless**, and **`nix-shell` may install anything needed**. Explicitly do
**not** use `nix profile install`; use `nix-shell -p <pkg> --run '…'`.

```bash
# Containers (verified: overlay storage, subuid 100000:65536, seccomp override works)
nix-shell -p podman --run 'podman run --rm --security-opt seccomp=unconfined <image> ...'
nix-shell -p podman --run 'podman build -f docker/integration/Dockerfile.guix -t shall-it-guix .'

# Rust toolchain for cargo (host cargo is 1.98; CI pins 1.97.1)
export LD_LIBRARY_PATH=/nix/store/lw86vj9kfwgvlivg3p8qg2nibpmhmsl9-zlib-1.3.2/lib
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export RUSTFMT=/nix/store/k2hwcha2p1fy0v0py34vq3jyvwqk339p-rustfmt-1.97.1/bin/rustfmt

# shellcheck (CI parity: -S warning)
/nix/store/qbz6d105ngpkwxlvxd303lafwavfpz8q-shellcheck-0.11.0-bin/bin/shellcheck -S warning \
  scripts/*.sh docker/**/*.sh .githooks/*

# Deny gate
nix-shell -p cargo-deny --run 'cargo deny check advisories'
```

The pre-commit hook (`cargo fmt -- --check`) **needs `PATH` and `RUSTFMT` above** or it fails
with `no such command: fmt`. `git commit --no-verify` is the documented bypass when you mean it.

Long commands must be backgrounded and detached (`setsid nohup bash -c '…' >/tmp/x.log 2>&1
</dev/null & disown`); a plain `&` dies when the tool call times out. Container image builds
(Rust compiled in-image) take 20–40 minutes on this host; the machine cannot profitably run two
at once (11 GB RAM, load spikes to 25 with two cargo builds).

## Verification chain (repo rule)

`cargo build --all-targets` → `cargo test --no-fail-fast` → `cargo clippy --all-targets
--all-features --locked -- -D warnings` → `cargo fmt -- --check` → `scripts/unix-check.sh`.
Never claim a step you did not run. The suite is one target, `suite`, built from `tests/main.rs`
(`autotests = false`); a new test file is invisible until it is a `mod` there.

**The suite is ~90 seconds, not twenty minutes.** `3049 passed; 0 failed` in 81.71s. The
twenty-minute figure is the container matrix, not the Rust suite.

**`scripts/lifecycle-floor.txt` is read by a Rust test** (`the_stated_lifecycle_coverage_matches_
the_ratchet_tests`), which checks README.md's coverage table against it. A "shell-only" diff that
touches that file is not shell-only, and the suite is owed. It caught exactly this on
2026-10-26: `aa970f12` raised the guix floor and called the diff shell-only; the next suite run
was red on the README table lagging it. Fixed in `ac797219`.

## Issues — all of them exist and are posted

Open, in order of what unblocks most:

| # | Subject | State |
|---|---|---|
| `#128` | `Advisories and licences` red: three `zstd`-chain advisories | **Closed** — `432dd20e`, verified |
| `#113` | guix section floor + guix/slackware `go` | **Closed** — `aa970f12` |
| `#129` | Four legs red on `check health (rc=2)`, no backend named | **Closed** — `cee823ae` (the backend was `bun`) |
| `#109` | apt legs install off baked lists | **Fix landed** (`5bf94cfc`), live ubuntu leg running |
| `#112` | native sweep family: installed packages missing from `list` and PATH | **`C7` built (2026-10-31)** — the install that converged the machine can no longer happen; the next native run answers the lister question with no harness change |
| `#130` | `handle_schedule` edits a line then converges the whole machine (the `C7` class on the sibling verb) | Open, untouched — filed this session; needs its own ruling (resource-scoping, not package keys) |
| `#100` | void leg's uv/yarn/gem/github listing failures | Open; `#112`'s delivery evidence lands on the next run |
| `#25` | stale umbrella | Open, untouched |
| `#111` | fan-out bounded by core count | Open, untouched |
| `#124`, `#94` | auto-declare on use | Open, untouched |

Decision register: **0 OPEN, 0 BUILT NEVER RULED**, **239 ANSWERED**, 2 PARKED (D15, D16), 1
DEFERRED (Q26), 2 HALF RULED (Q29, J4). `C7` (2026-10-27) is the new ANSWERED entry: *install and
uninstall converge exactly what they name* — rule in II.70, rationale in V.220, supersedes the
converge half of Q34/V.136 (the failure-naming half of that entry survives). None gate this work
other than `C7`'s build. Any ruling ships in the same commit (`docs/spec/decisions.md`, plus
`target-state.md`/`why.md` if it is a rule) and `scripts/decision-count.sh --check` is the gate.

## What was measured this session

### `#113` — the guix leg, verified by a local podman run

The full image built (rc=0) and the harness ran green: `pass=332 fail=0 soft=9`. Every section
the issue quoted as breached is back at or above floor — `The guard` 3 (was 2), `Git history +
rollback` 8 (was 3), `rebuild` 6 (was 3), `Chains and the per-host lock` 17 (was 15), `Command
surface` 70 (was 68), `Coverage audit` 5 (was 4) — and the real-lifecycle ratchet measured 4,
recorded with the reason.

**The slackware half was the harness, not Shall.** Shall's own WARN already said the `go` line
was not what failed and named `slackpkg:clucene`; the harness titled its FAIL after the subject
it happened to be driving. `classify_install` now extracts Shall's `` while applying `X` `` line
and names the real culprit in all three defect verdicts, in the container harness and its
Windows twin. **The next slackware nightly is the live proof** — nothing in that half was
verified against a real manager run.

### `#129` — the four legs' sick backend was `bun`

The harness's evidence half (`99540c13`) named it on the first post-fix run: `bun — answered with
1 line(s) this parser does not recognise, the first being /home/shall/.bun/install/global
node_modules (0 installed)`. Newer bun appends the word to its npm-style count;
`bun_header_count_is_zero` read only a bare `(0)`, so every empty-bun machine read as unreadable
and the psresource-class promotion made it CRITICAL. Fixed by stripping a trailing ` installed`.
Test written first against the arch log's bytes, watched it fail as
`Unrecognised { data_lines: 1, sample: … }`, then pass. The empty-listing module had no bun row
at all; it has one now.

### `#112` — the native family has ONE cause, and the artifact proved it

**Read the artifact first** (that was the issue's own instruction, and it is what settled it):

```bash
mkdir -p /tmp/opencode/ev112 && cd /tmp/opencode/ev112
# `gh run download` needs a git repo for the repo context, and it is NOT the run's repo — pull
# the artifact zips through the API instead (the macOS one is only 3.7 KB):
A=$(gh api repos/SYKhayyat/Shall/actions/runs/40353656991/artifacts -q \
    '.artifacts[] | select(.name=="native-sweep-evidence-macos") | .id')
gh api /repos/SYKhayyat/Shall/actions/artifacts/$A/zip > macos.zip && unzip -q macos.zip
# …and the per-job logs (slow — 1.7k and 2.7k lines; run them detached):
gh run view --job 121680207727 --log > macos-job.log      # macOS
gh run view --job 121680208082 --log > windows-job.log    # Windows
```

**What the logs say.** `shall -y install cargo:hexyl` produced a plan of **one change, and it
was not the canary** — macOS ran `backends: brew` + `✓ [brew] rustup (5305ms)`; Windows ran
`backends: winget` + `✓ [winget] ImageMagick.ImageMagick (10588ms)`. Across the family, every
canary install converged the adopted model instead: dotnet → `brew pkgconf` / `choco sbt`, web →
`brew maven` / `choco wixtoolset`, yarn → `brew perl`, pip → `winget LLVM.LLVM`, npm →
`winget Microsoft.AzureCLI`, helm → `winget Amazon.SessionManagerPlugin`, choco → `winget
Microsoft.PowerShell`, go → `winget Microsoft.PowerShell`.

**Why.** `shall install X` is "write the line, then converge the whole configuration"
(`src/verbs/packages.rs:52`, P1/S15). By section 12 the sweep's own `adopt` has made the config
*be* the machine — 43 adopted packages on macOS, 103 on Windows — so the install's plan was a
whole-machine converge, and the canary never reached it: on macOS cargo was reported not
installed by the lister, on Windows cargo's listing came back empty. The harness read that run's
exit 0 as "installed for real" and then failed the listing and PATH checks against a package
that was never installed.

**So it is one bug, not ten lister bugs** — and the lister question the issue was filed to ask
was never asked, because the install never installed. That is what `C7` builds, and the next
native run answers the real question with no harness change.

### `#109` — the apt mechanism, measured

Over HTTP, at the URLs a stale list resolves to, inside a locally built ubuntu image:

```
206  curl_8.5.0-2ubuntu10.16_amd64.deb   <- the current candidate
404  curl_8.5.0-2ubuntu10.14_amd64.deb   <- superseded, GONE from the pool
404  curl_8.5.0-2ubuntu10.11_amd64.deb   <- superseded, GONE from the pool
206  curl_7.81.0-1ubuntu1.30_amd64.deb    <- an older RELEASE, kept
```

The premise holds for the current suite's point revisions; every version the image's own baked
lists resolved fetched cleanly (15/15). CI builds the image fresh and the lists are fetched at
the top of it: on the ubuntu leg of `40353618389`, `apt-get update` at 15:53:05 and the first
install at 15:59:07 — six minutes old, leg running 34 minutes after. **No failing run has ever
been attached to this issue, and none is attached now.**

The fix (`5bf94cfc`) refreshes before the FIRST install — not before the service install the
issue's candidate named — and is best-effort: a refresh that fails because the mirror is down
must not red the leg for a precondition, which is this issue's own complaint arriving through the
fix. A failed refresh is a soft and the leg says it is running on the image's baked lists.

### `#112` — the native sweeps, evidence commented

The dispatch run `40353656991` (macOS job `121680207727`, `pass=257 fail=15 soft=25`) put the
install excerpts in the log for the first time, and they are the shape of the family: `cargo`'s
canary install reports `Status: SUCCESS` with `backends: brew` and `✓ [brew] rustup`, then the
listing says the manager is not installed at all. Two hypotheses the uploaded artifact
(`native-sweep-evidence-macos`) can separate: the bootstrap did not deliver (a `brew install
rustup` formula provisions nothing until `rustup default` runs), or the harness read the wrong
page. `gem`'s listing shows the system ruby's gem set without the canary, which reads as the
toolchain mismatch. The `dir` coverage failure is fixed (`ac797219`).

## Suggested order from 2026-10-27 (superseded above)

1. **Build `C7`** — the work order is #112's `issuecomment-6475969648` and the outline is the
   `### C7` section at the top of `docs/spec/plan.md`; the tests are reproduced in the #112
   comment. Tests first, watched failing against the unmodified code, then the mechanism, then the
   deletions. The ruled shape (no flag, no second verb, shell chaining) was chosen against the
   codebase's own shapes (`upgrade` narrows by what you name, `adopt` takes the backends you
   name) — do not re-litigate it without the owner.
2. **`#130`** (the `handle_schedule` sibling): needs its own ruling before any code, because
   resource-scoping is a design choice, not a flag reuse. Not a blocker for `C7`.
3. **Read the ubuntu leg's result** (`/tmp/opencode/harness-ubuntu.log`, a local podman run of
   the whole leg with the refresh in place) and close `#109` with it.
4. **Slackware's live proof** for `classify_install`: the next nightly, or a local podman run of
   `Dockerfile.slackware`.
5. `#100`: read the next void run — `#112`'s delivery evidence lands there.
6. **After `C7` lands, dispatch a native run** — `#112` closes on that evidence, either way: the
   family goes green, or a narrowed install fails loudly and names the genuinely broken manager.
7. The macOS ratchet measured **10 ≥ 8** and printed the ratchet-up `sed`; it is NOT recorded,
   on the two-run caution the windows row carries. Decide it with the next macOS run.
8. `#109`'s sibling worth checking, not yet measured: dnf's metadata expires in 48h, so a fedora
   image's baked metadata is the same class one day older.

Anything you find that is not already an issue must get one, with file:symbol evidence, a
reproduction or the test gap, and a proposed resolving test — that is the rule the owner set
this session.

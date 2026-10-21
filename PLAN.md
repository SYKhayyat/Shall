# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58, #69, #70, #71, #34, #37, #75, #76, #77, #78, #35, #99, #97, #102, #98, #91, #86, #85, #81, #83, #84, #79, #82, #87, #95, #103, #88, #36, #55, #101, #105, #106, #107.

## SKIP — duplicates of one event, do not re-work
- #32 DUP of #23+#24; #48 DUP of #32; #49 DUP of #33; #45 DUP of #26 (same file:line, same fix).

## Phase 1 — Foundations first (primitives others compose into)
- [x] #69 templated content (writeText analogue with per-user substitution). (Medium — unblocks #73)
- [x] #70 secrets compose into templates (${secret:} flow). (Medium)
- [x] #71 per-user home layer (manifests, ownership, idempotent dirs). (Medium)
- [x] #34 converged fast-path duplication → route through Phase::all. (Medium)
- [x] #37 proven-by-default polarity flip. (High)

## Phase 2 — Safety Criticals/Highs
- [x] #75 download truncates live PATH binary → temp+rename. (Critical)
- [x] #76 symlink guard bypass via parent. (High)
- [x] #77 watch blind to profiles/vars. (High)
- [x] #78 single info() failure aborts whole plan. (High)
- [x] #35 Reaped self-attestation token. (High)
- [x] #39 harness half-totem (mutation survival). (High — absence/restraint family shipped as
  `0f9c5e5`; the ISSUE STAYS OPEN, because section 16's ~50 content claims are the larger half
  and are named in the issue, not here)
- [x] #40 two oracles that cannot fail. (Medium — **both shipped, and the ISSUE STAYS OPEN
  deliberately**, because what is left of it is in the plan and not in the product. Verified in the
  tree for this review: `deny_unstubbed`/`unstubbed_registrations` in `core/executor.rs` with
  `tests/an_unanswered_command_is_a_defect_tests.rs`, and `latency::UNMEASURED_TOKEN`
  (`shall-latency-unmeasured:`) emitted from every path that declines to measure, with a gate per
  path. The follow-up it names — 66 suite tests whose `execute()` calls nothing answers — is
  measured and recorded in `docs/spec/plan.md` Tier 14, so closing the issue would lose the only
  register of it)
- [x] #99 a harness section that stops running is invisible: `crash/groupkill` lost 10 checks on
  ubuntu and the run still went green. (High — there is no floor on *checks run*, which is the
  exact mirror of `CAUGHT_FLOOR`, and three of the five properties only the matrix can verify are
  driven by that section. **BUILT, both halves** — the per-section tally in 5794ac3 and the floor
  in `scripts/section-floor.txt`, an absolute count per host class per section, measured off four
  runs of the matrix. A `soft` excuses a shortfall only against a dated line carrying both the
  count it covers and the text this run printed, so the ubuntu run in the finding fails.
  **NOT the Windows/macOS half**: neither prints a section tally, so the gate is in force on
  neither — see `docs/spec/plan.md` Tier 14c)
- [x] #96 CI: the 8 distro integration images are red on the container harness's `dir:` check (a
  check that never observes what it asserts). (**BUILT — and it was one missing line of harness,
  not a product defect.** `gone_ok "the dir is gone from disk" dir-dst` had no `witness dir-dst`
  anywhere, so the teardown could never be scored and all eight legs reported
  `the declared dir is on disk` PASSING beside a `gone_ok` that refused. The check that "never
  observed what it asserted" was `gone_ok` doing its job)
- [x] #97 CI: `Advisories and licences` and the MSVC build are red on `main`; the MSVC one means
  the `#71` Windows code has never been compiled by anything. (High — **BUILT, both halves**.
  MSVC: fourteen compile errors, and the local chain can now type-check that platform at all.
  `Advisories and licences`: `RUSTSEC-2026-0285` against `rustls` 0.23.43, three hops down
  through `reqwest` — **`cargo update -p rustls` to 0.23.45**, one package, and deliberately
  **no `deny.toml` entry**, because that file calls an ignore "a build made green by a decision
  rather than by a fix". Same declared MSRV, so the MSRV job is untouched)
- [x] #101 CI: both MSVC test binaries die with `0xc0000005 STATUS_ACCESS_VIOLATION` mid-run, and
  the log cannot say which test crashed. (High — **BUILT. One null out-parameter, in one
  function, and it killed both binaries.** This is what the fourteen compile errors were hiding:
  the crate compiles and then the process dies, so `cargo check` cannot answer it. The job runs
  without `--nocapture`, so the crash discards its own evidence — **one**
  `--test-threads=1 --nocapture` run should name the test outright. I first reported the two
  `FAILED` lines here as two failures; the exit codes showed the process died before libtest
  could flush them, so they may only have been in flight.
  - **THE `#102` RERUN HAS NOW NAMED IT, so the 2026-09-30 DEFERRED verdict no longer holds — and
    the crash is deterministic, which is what makes it tractable.** The serial
    `--test-threads=1` rerun dies in the same *module* as the parallel run:
    - `--lib`: **268 of 2222** reported, then died. Last module `app::adopt::tests`; the next is
      `app::apply`, whose first test is `dotfiles::tests::outside_home_uses_the_selected_user_home`.
      That calls `Account::current()`, whose Windows half is `LookupAccountNameW` →
      `ConvertSidToStringSidW` → `RegOpenKeyExW`/`RegQueryValueExW` on `ProfileList` →
      `ExpandEnvironmentStringsW`. **The first raw-Win32 call the lib binary reaches.**
    - `--test suite`: **480 of 723** reported, then died. Last module
      `ansi_is_for_terminals_tests`; the next is **`argv_drift_tests`**, which reports nothing and
      has exactly **one** test, `every_subcommand_shall_invokes_still_exists_upstream` — the only
      test in the suite that shells out to dozens of real managers through `Command::output()`.
    - **The `running 0 tests` lines in that log are the DOC-TESTS target, not the lib.** Recorded
      because reading them the other way round makes the lib look like it died before starting,
      and that misreading is a large part of why this looked unnameable.
  - **THE FIX, and it is one line of the call.** `resolve_named` passed
    `std::ptr::null_mut()` for `LookupAccountNameW`'s `peUse`. That parameter is a plain
    out-parameter and not an optional one: the API writes the `SID_NAME_USE` it chose **on
    success**. With two null buffers the first call fails (`ERROR_INSUFFICIENT_BUFFER`) before it
    reaches that write, which is why the sizing call survived and the second — the one that
    succeeds — killed the process. **Reduced to eleven lines of raw FFI on a Windows host before a
    line of this tree was touched**: that sequence segfaults (`0xc0000005`, the same code CI
    printed), and the same sequence with `&mut pe_use` returns `SidTypeUser` and the SID below.
  - **Why one null is both binaries.** Every caller of `Account::current()`/`Account::resolve()`
    died with it, and there is one on each crash path: `--lib` dies in
    `app::apply::dotfiles::tests::outside_home_uses_the_selected_user_home`, whose first statement
    is `Account::current().unwrap()`; `--test suite` dies in `argv_drift_tests`, which drives
    `install`/`remove` on every backend present here, and whose `link`/`dir` backends resolve
    `@target=` through the current account. The two names #102's rerun produced were both right,
    and they were one bug.
  - **A second defect was hiding behind the first**, found while proving it:
    `ConvertSidToStringSidW` returns `LPWSTR`, and `resolve_named` and `sid_text` each read it with
    `CStr::from_ptr`. That stops at the first `0x00` byte — the high half of the first `u16` — so
    every account's identity read back as `"S"`, `ProfileList\S` did not exist, and
    `Account::current()` answered *"could not find a home directory for account …"* instead of
    crashing. **A string that is wrong reads exactly like an answer**, which is why it survived to
    the next call and why the whole call chain looked correct in review. `wide_to_string` is the
    one reader now, and `the_current_accounts_sid_reads_back_as_a_sid` is its control:
    `assert!(sid.starts_with("S-1-"))` is not a string a half-read buffer can produce.
  - **The guard around the sizing call was inverted**, fixed in the same change:
    `first != FALSE && GetLastError() != ERROR_INSUFFICIENT_BUFFER` carried on after a lookup that
    had failed for an unrelated reason (an unknown account is `ERROR_NONE_MAPPED`), with a
    zero-length SID and a zero-length domain buffer. It is `first != FALSE || …` now, which is the
    only outcome a null-buffer lookup is allowed to have.
  - **`panic = "abort"` in `[profile.release]` is not the cause, and that was checked rather than
    reasoned about.** A two-test crate under the same profile setting prints its `failures:`
    section and exits 101 — Cargo forces `unwind` for test targets — so the missing `panicked at`
    lines were a dead process rather than a silenced one, exactly as the issue said.
  - **Both commands the last comment asked for were run, on Windows, and both now report:**
    `cargo test --lib app::apply::dotfiles` is 3 passed where the process used to die, and
    `cargo test --test suite argv_drift_tests` names a result instead of nothing at all.
  - **The lib-side Win32 fix also type-checks on the cross target**, with
    `cargo check --target x86_64-pc-windows-gnu --lib` and `RUSTFLAGS=-D warnings`, using the Nix
    mingw C toolchain. It supplies the required `SID_NAME_USE` output buffer, checks the sizing
    error correctly, and decodes the UTF-16 SID returned by `ConvertSidToStringSidW`. The cross-target
    `--tests` check passes with warnings denied. That is
    compile evidence from a Linux host, complementing the Windows runtime run above.)
- [x] #102 CI: the main test step discards a crash's own evidence, on every platform.
  `.github/workflows/ci.yml:363` ran for **every** row of the build matrix with no `--nocapture`
  and no `--test-threads`, so a binary that dies mid-run never reaches libtest's failure report —
  run `36598780716` logged zero `panicked at` lines in 78KB and no `failures:` section, which is
  what left #101 undiagnosable from CI. (High — **BUILT**: one failure-only step after `Run tests`
  on the whole matrix, `--test-threads=1 --nocapture`, gated on `failure() && matrix.native` so a
  green job pays nothing and a cross row that failed at *build* does not try to test a binary it
  never produced. `every_build_row_can_name_its_own_crash` guards it, and the control is the main
  step: if `Run tests` ever goes serial, the check would be satisfied by the step it is supposed
  to follow)
- [x] #98 CI: the `why.md` unattached-rationale ratchet is red on `main` — 53 entries against a
  ceiling of 52, and it reds `Build for x86_64-unknown-linux-gnu` on every push. (High — **BUILT,
  and the five were the cheapest kind there is**: `V.129`, `V.137`, `V.138`, `V.143` and `V.149`
  each already named the rule they explain — II.2, II.9, II.11, II.7, II.7c — and those rules were
  already carrying the reasoning in their own prose **without the citation**. 53 → 48, and
  `UNCITED_CEILING` re-pinned 52 → 48 in the same change, which the ratchet's second assertion
  demands)
- [x] #100 the `void` leg's failure list is a sample of the network. (**BUILT — and the issue's
  own log contradicts its proposed mechanism, which is why the fix is a different one.** Reading
  the run rather than the summary: **four** backends fail in both runs (`github`, `pnpm`, `uv`,
  `yarn`), not two, and each printed `PASS <be> installed <pkg> for real` *first*. So routing the
  per-backend verdicts through `CLASS` could not be the mechanism — `classify_install` runs only
  when the install **failed**, and these returned 0. **The rc was the untrustworthy thing, not the
  backend:** `uv tool install pyjokes` returned 0 having delivered nothing, a registry that
  answered and served nothing, silently.
  - **And the removal half was vacuous in the same breath.** `nok "$be: $ctok is gone from list"`
    passes when the grep finds *nothing*, so on a package that was never installed it is a green
    line. Five checks ran per backend and three of them scored a package that was never on the
    machine — which is the mechanism behind the issue's own complaint. The four systematic failures
    were indistinguishable from the six that rotated **because they were the same shape**.
  - `install_delivered` asks whether the package arrived (its listing reports it, **or** its binary
    is on PATH) before the removal half is scored. A backend whose listing cannot show the package
    (`cabal`) counts as delivered — the listing is the wrong witness for it — and that is asserted
    as the *control* against the failing case: same empty listing, opposite answers, differing only
    in whether the lister can speak for the backend.
  - Undelivered ⇒ one loud `soft`, `be-life-unmeasured` (measurable-in-principle, not coverage
    *lost*) and `be-life-partial`, non-zero return.
  - Its own small function because `lifecycle` is far past `lift`'s runaway guard. Four cases
    driven; **mutation watched** — `return 0`, the pre-fix behaviour, reds two of them.
  - **The refactor's own regression, and the best argument this file has for `section-floor.txt`:**
    `install_delivered` first returned 0 the moment the listing matched, so `assert_binary_reachable`
    never ran — five checks per backend, gone, on five hosts (alpine 34→29, fedora 44→38, opensuse
    40→34, arch 56→45, void 37→15). **Nothing about that change looked like a deleted check:** the
    tests were green, the predicate count was unchanged, and the stated intent was to make the leg
    *stricter*. It asks "did anything arrive", not "how many questions did it take", so both witnesses
    are now scored and the answers OR'd after.
  - **Two floors re-pinned, and the number is the point rather than the problem:** arch 56→53 (`go`)
    and void 37→18 (six backends), each with a dated excuse naming the `soft` this run must print.
    Void's old excuse (`appimage`, down to 33) is *replaced* rather than added — a line carries one
    excuse, and the leg now has a bigger one.
  - **Still open behind it:** *why* those registries answer 0 and serve nothing. This makes it one
    visible class rather than five half-vacuous checks, and deliberately does not guess. — ISSUE #100)

## Phase 3 — Correctness Mediums
- [x] **Gate names cited anywhere are not checked for resolving, while `V.n` citations are.**
  (Low — **BUILT, and it found NINE the day it was written**, which is the answer to "would
  anything notice the next one". `tests/a_gate_name_cited_anywhere_resolves_tests.rs`, the twin of
  `every_citation_of_a_why_entry_resolves_to_one`.)
  - **Five shapes, all real:** a gate **renamed** with its citation left behind
    (every_os_native_backend_sends_the_argv_its_manager_expects, cited from two source files and
    defined nowhere); **truncated** forms of gates that exist under longer names
    (every_ledger_prefix_is_claimed, no_unbounded_command_holds_the_lock_for_its_lifetime,
    an_offered_archive_has_an_opener, a_failing_command_names_its); a name **split across a
    line**, so grepping it found nothing; and one that had **lost a character at the wrap**
    (`..._still_knows_its` + newline + `module`, where the fn is `..._still_knows_its_module`),
    which joining does not recover.
    **These names are set as plain text on purpose.** A backtick span is what makes the scanner
    read something as a citation of a gate, so quoting a name the gate *reported* would have this
    entry fail the gate it is describing — which is the correct behaviour and not a bug to
    special-case. A finding about dangling citations is the one place a dangling name belongs.
  - **Two shapes of the SCAN were wrong before they were right, and both are commented where they
    bit.** Joining every line break inside a backtick span merged a wrapped sentence into invented
    names — apt_like_coresetsorphan_dry_run — so a break is joined only where it follows an
    underscore — which is where a wrapped identifier actually breaks. And skipping only whitespace
    stopped at the `///` that opens a doc comment's continuation line, inventing four more. Same
    failure both times: **a checker that cries wolf gets switched off**, and each was caught by
    reading the report rather than by the assertion.
  - **Four name forms count as defined** (test `fn`, `mod`, file stem, file stem minus `_tests`),
    because this corpus uses all four and a scanner accepting one reports every gate in the other
    three as dangling. `EXEMPT` holds fourteen historical citations with their corpus, may only
    shrink, and the assertion says so.
  - **The mutation, and the wrong first attempt.** Renaming a gate with **no citations** is invisible
    to any citation-based check — worth knowing rather than assuming. The watched mutation renames a
    gate that *is* cited, and the gate names the file and the dangling citation.
- [x] #91 planner template_needs_update compares raw source to rendered target (sibling of the #69 read-back fix). (Low; **BUILT by deletion** — the issue named two options and said not both. `in_effect` is the single authority and it renders before comparing, so the planner's copy went: 24 lines, and no behaviour changes because `link` installs no `Queryable` and the branch never ran. `the_planner_never_reads_the_filesystem` keeps it gone, with the deleted code as its own control)
- [x] #86 zip unpacked-size sum can wrap past bomb bound. (**BUILT** — the zip branch called
  `.sum()` where the tar branch walked with `saturating_add`, and `sum` for integers **wraps in a
  release build**, so members declaring `u64::MAX` and `2` totalled `1` and the bomb check said
  yes. Both branches now go through one named rule, `add_unpacked`, because the defect was never
  the arithmetic — it was the arithmetic being written twice four lines apart.)
- [x] #85 HTTP pool check-then-insert race. (**BUILT** — `get`-then-`insert` across a window the
  width of `build()`, so sixteen concurrent asks for one policy built up to sixteen clients and
  kept one. Now `POOL.entry`, which holds the key's slot across the build. Measured, not argued:
  the old shape built **2, 7 and 7** clients for one policy across three runs of the new test,
  and exactly 1 with the fix.)
- [x] #81 Mutex across --help. (**HALF BUILT, and the other half is not a defect.** The wait is
  intentional singleflight — one `--help` per program per run, the shape `InstalledListings::once`
  and `VARS_MEMO` already use — so replacing the std lock with a tokio one would not remove the
  wait, only change which executor thread stalls. **What was a defect is the poison path**: both
  arms answered `probe(…)` on a poisoned lock, and a `Mutex` is poisoned *permanently* by the one
  panic, so a single panic anywhere in the process switched the cache off for the rest of the run,
  silently. Now recovered with `into_inner`, and `a_poisoned_cache_is_still_a_cache` poisons both
  locks for real and asks a question the cache can answer from memory.)
- [x] #83 vars JSON detection fragile to BOM/banner. (**BUILT** — the rule was `starts_with('{')`
  after `trim()`, and a BOM is not whitespace, so valid JSON behind one was read as `name = value`
  lines and came back as an error about a line the user never wrote; a banner before the document
  did the same. Now: BOM stripped, then a **line-anchored** scan for the first JSON object.
  Deliberately NOT `parsers::json_document` verbatim — that also tries the first `{` *anywhere*,
  which would read `labels = {"a": {"b": 1}}` as JSON and invent a variable called `a`. That
  regression is pinned by a test, and the mutation that removes the anchoring turns it red.)
- [x] #87 copy_over mishandles a dir-symlink on Windows. (**BUILT, and the one part that could be
  watched here was the part a naive fix would break.** `copy_over` asked `symlink_metadata` and then
  always reached for `remove_file`; `symlink_metadata` does not follow the link, so a **directory**
  symlink answered "not a directory" and went to the file form — right on Unix, and `Access is
  denied` on Windows, where `remove_file` cannot delete one. The symlink case now goes through
  `remove_deployed_path`, the async face of the `remove_by_kind` that `force_remove` already uses,
  so the Windows symlink rule has **one home here instead of three** — which is what this file's own
  comment at `remove_by_kind` predicted ("two copies of *which* removal call a symlink needs is how
  one of them ends up with the Windows arm and the other without it"). **And not `remove_by_kind`
  outright**, because on a real directory it calls `remove_dir_all`: a straight swap would turn "a
  directory in the way is a different fault" into an emptied tree and a successful copy. That
  refusal is now a test, and it is the mutation that was watched — the naive swap reds it.
  **Said so: the Windows symptom itself cannot be reproduced on this host** (on Linux the old arm
  worked), so the symlink test characterises the shape rather than watching the fix fail; what is
  verified here is that replacing a directory symlink leaves what it pointed at alone, and that a
  directory in the way is refused with its contents intact. CI's Windows job is the instrument for
  the rest. — ISSUE #87)
- [x] #84 pre-batch prior_state fan-out width = batch size. (**BUILT — and the gate that should
  have caught it had a hole of its own.** The prior-state read was fanned with
  `.buffered(members.len().max(1))`: no literal, no config field, so `fanout_cap_reads_the_setting
  _tests`, which detects a cap that ignores the setting, walked past it — its test for that was
  *"the argument starts with a digit"*. That is a way of saying *a literal*, and a literal is only
  one of the two ways a width can ignore the setting; the other is the width of the collection
  being fanned out, which gets *wider* as the batch grows — four hundred listings, each of which
  can spawn a manager process, at the one call that runs **before anything is touched**. Now
  bounded by `config.max_concurrent` (`sync` already derives it from `max_parallel`), and the
  gate's predicate now catches both shapes with a test per direction.
  **The other half is a mutation nobody would think of:** the result is indexed by member position
  (`priors[i]`), so it is one answer per *slot*, not a set — `buffer_unordered` compiles, runs, and
  puts another package's history in every slot, so `rollback` restores the wrong thing with nothing
  to fail. Width lowered, `buffered` kept, and a source check plus a control pins the ordering.)
- [x] #80 probe storm. (**BUILT, and the half that needed no judgement call is the half that
  was the defect.** The issue offers two fixes: cap the fan-out with `max_parallel`, or cache probe
  answers per run. **The cap half is already ruled at the site and the gate now enforces it** — the
  probe fan-out's width is the number of probes the user declared, and its own comment says why
  (*"a user who declared four property probes asked for four questions, not for them to be
  rationed"*), which `fanout_cap_reads_the_setting_tests` (extended by #84 to catch a width taken
  from the collection being fanned out) would otherwise have flagged as a literal-free width. The
  `join_all` there is deliberately **not** in that gate's `CAPS`, and the reason is written in the
  gate rather than left at the call site. **The caching half was a real defect and is fixed:** a
  probe's `{name}` is substituted into its **template**, never into its argv, so `npm prefix -g`
  cannot depend on the package — and `info` runs one subprocess per package to ask it anyway, so
  twenty packages cost twenty `npm prefix -g` to learn one prefix. Now `run_output_once_per_run`:
  keyed by the whole argv, singleflight, **failures not cached** (the `once` rule), carrying the
  round in the answer so an answer taken across a mutation is stale on arrival, and dropped by
  `forget_all()` — which `forget_run_scoped_answers` already called, so there is no second
  invalidation to forget to call. Measured as a count, not a clock: three `info` calls issued the
  probe three times, now once. **And the whole cost of `info` over N packages is now one listing
  and one subprocess per distinct probe argv, for the run** — reading `GenericQueryable::info`
  says so: `installed_listing()` (memoised per backend per run), then the probes (memoised per
  argv per run), and nothing else in it spawns anything. The invariant that makes the memo
  sound — no probe argv may name the package — is a gate over every shipped row, and planting
  `{name}` in npm's argv was watched to fail it. **The memo went into `core::installed` rather
  than `core::executor`:** written in the executor it broke
  `no_file_grows_past_readable_without_a_written_reason`, which sits at **exactly** its recorded
  3,150-line ceiling, and the answer to "where does a second run-scoped memo live" was already
  written in that file — beside `by_backend` and `essentials`, so it inherits the generation and
  `forget_all` instead of inventing a second staleness policy. **Said so rather than implied:**
  the round-stamp check is kept because `once` keeps it, and dropping it leaves every test green,
  so nothing here claims it isolated. **Remaining:** the *width* of the probe fan-out
  across packages is the outer fan-outs' business and those are capped (`max_concurrent`,
  #84), so what is left here is measurement on a machine with real managers, which this host does
  not have. — ISSUE #80)
- [x] #79 forget_all wipes all caches. (**BUILT, as a setting rather than a reversal.**
  Y6 binds *"any mutation drops it, on disk as well as in memory"*, and that stands: the new
  `listing_invalidation` key **defaults to `all`**, which is the ruling. `mutated_manager` is the
  lever the ruling's own reasoning implies — a user who has measured their machine can move it —
  and it forgets the listings of the backends sharing the mutated manager's lock
  (`app::stale_lock::lock_key`, the table that already answers "which manager", so **no
  registration step and nothing to forget to wire up**) plus that family's per-backend cache
  files. **Only `run_exclusive` narrows**, because it is the only mutation path handed the
  *manager*; the other two are handed the program, which XIII.12 makes a different string for a
  user-defined backend, and they keep forgetting everything. `essentials` — the set that refuses
  removals — is cleared whole under both scopes, as is the `PATH` memo. No generation bump in the
  narrow path, because it is global and would invalidate the listings the scope exists to keep.
  Ruling, the declined change-token destination and the reasoning are in `decisions.md` under Y6;
  II.19 and V.120b record the rule. Seven tests: the lock family goes together and an unrelated
  manager does not, the essential set still goes, the family's cache files are deleted and the
  others are not, plus end-to-end both ways through `run_exclusive`. — ISSUE #79)

- [x] #82 batch deadline. (**BUILT — and what was built is not the fix the issue asked for,
  because both failure modes it describes are unreachable with the defaults.** The formula is
  that is worth more than the fix the issue asks for.** The formula is
  `node_timeout * names.len().clamp(1, 16)` with `node_timeout = 300s` and — the part the issue
  does not mention — **`total_timeout = 3600s`, wrapping `execute_internal` whole**
  (`transaction.rs:454`). So: 1 package gets 5 min, 11 get 55 min, **12 get 60 min, and 16+ get
  80 min — which the transaction-level timeout has already killed.** The issue's second claim (*a
  100-package batch times out spuriously, bisects, and repays apt ten times*) needs an 80-minute
  node deadline to fire inside a 60-minute run, so it cannot happen; and the first claim (*a
  generous per-node deadline pins the wave and holds the manager lock*) is bounded by the remaining
  total budget, not by `16 × 300s`. What is actually true is duller and worth fixing: **the clamp
  is dead above 12 packages**, a magic number with no relationship to the budget it is supposed to
  fit inside, and nothing tests any of it. The buildable half needs no ruling — extract the
  deadline as a named function, cap it at `total_timeout` (**behaviour-preserving, because the
  transaction timeout already dominates**), and pin that it is monotone in the batch size and
  reachable for every batch the clamp admits. The part that *does* need the owner is the one the
  issue is really about: **how long a hung manager should be allowed to hold the lock**, which is
  a user-visible timing choice with no measurement available on this host. **Owner ruling,
  2026-10-01: expose it as a config key and change no default** — so `manager_command_timeout_secs`
  (default 300, the value `node_timeout` always had with nothing to raise it), and
  `node_deadline()` capping the scaled budget at `total_timeout`, which changes no outcome and
  makes the two constants' relationship a fact instead of an accident. Four tests in
  `core::batch`: the per-package budget, monotonicity, the cap across batch sizes, and **that the
  cap did not swallow the shape** (a fix that flattened everything to an hour would pass the cap
  test and give one package an hour). — ISSUE #82)
- [ ] #25 lifecycle jobs on distro containers.
- [x] #54 dirty-host fixtures: the Windows/macOS native sweep now creates a unique run root before
  deriving config, data, ledger, stall, and helper paths, cleans it on exit and signal paths, and
  has a source-level isolation control in `dirty_host_fixture_isolation_tests`.
- [x] #53 Nimble Windows lifecycle: the nightly Windows-native job installs pinned Nim 2.2.12,
  refreshes Nimble's index, and runs `integration-windows.sh nimble nimjson` through Git Bash.
- [ ] #109 CI: the apt legs install off the image's baked lists, so a mirror rotation reds the leg
  for the mirror's reason. (**Filed 2026-10-15 with a candidate patch and NO failing run behind
  it, and the issue says so itself.** `Dockerfile.ubuntu` runs `apt-get update` at build and keeps
  the lists; nothing refreshes them before an install, so an install off a suite whose revision
  the pool has rotated away 404s — and `docker/integration/retry.sh` deliberately does not wrap
  the distro's own managers, so a mirror's fault surfaces as a red leg. The evidence for the
  *mechanism* is real (`audit/results/ubuntu-build.log:354` resolves `noble-updates` at run time);
  **the evidence for the failure is not**, and landing the fix would put an unproven precondition
  into the harness — the same shape of mistake as `#96`'s missing `witness`. **Not landed here:
  there is no Docker on this host**, so neither the harness nor the patch can be executed, and the
  issue's own first step is to attach the apt log. The patch applies cleanly to `c5d0547`
  (`git apply --check`), so it is ready for whoever has a container runtime.)
- [ ] #112 the two native sweeps are red on the same family. (**Filed 2026-10-18 off the nightly
  logs, because macOS native had no issue at all and Windows had two that name two symptoms.** Ten
  backends on Windows and eight on macOS fail the same pair — `list shows <pkg> (missing)` and
  `<pkg> is not on PATH and nothing said where it went` — after the harness watched the install
  return 0, which is a family rather than ten coincidences. Three other families are named in the
  same issue: the Windows `link` segfault — **VERIFIED as `#101` on this host, after the issue was
  filed, rather than inferred from the rc**: a worktree at `c5d0547` built with its own
  `CARGO_TARGET_DIR` and driven with the sweep's own fixture
  (`scripts/integration-windows.sh:1620`'s `link:/etc/hostname @target=/tmp/shall-it-hostname`)
  segfaults with **`rc=139`** on both commands the sweep runs, where current `main` answers `rc=2`
  (`drift … 1 to place`) and `rc=0` — the crash family that never converges, and `coverage: 1
  backend(s) have no path to a real lifecycle, over the ceiling of 0`, which names the count and
  not the backend. macOS's
  `installing a nonexistent package fails (expected non-zero, got 0)` is filed as a suspected
  *consequence* of the listing family, with the order of repair as its control.)
- [ ] #113 the `guix` leg's section floor is red. (**Filed 2026-10-18 off the nightly logs.**
  `RESULT pass=316 fail=9 soft=12`, and the most specific line is not one of the nine: **`section
  floor: this run credited fewer checks than container-linux-guix-local has before`**, naming six
  sections short — `Chains and the per-host lock` 17→15, `Git history + rollback` 8→3, `rebuild`
  6→3, `Command surface, executed` 70→68, `The guard` 3→2, `Coverage audit` 5→4. That is `#99`'s
  instrument firing on a leg nobody has read, and it says how much of the leg stopped being asked
  where no single `FAIL` does. Several of the nine read like a manager that installs nothing there
  (`nothing in this run was ever seen as 'lockfile-entry'`), so the floor may be a consequence —
  **the first step is to say whether `guix install` delivers on that image**, with `#100`'s trap in
  mind: an `rc` of 0 is not delivery. Slackware's single red is `go: … failed permanently` and is
  filed here rather than alone, because `go` is the one backend failing on three legs for the same
  subject (`arch`'s is in `#100`). **Not started here: it needs a container runtime, and there is
  no Docker on this host.**)
- [x] #115 the removal guard's undo door has never been tested. (**Filed 2026-10-20 off the
  nightly's own mutation output, and the finding is a family.** `enforce_own_work`
  (`src/app/sync/guard.rs:1371`) — the door a failed sync uses to undo what it just installed —
  has **zero callers in any test file**: a grep of `src/` and `tests/` returns its definition and
  its two production callers (`src/core/transaction.rs:1681`, `src/app/sync/mod.rs:1428`) and
  nothing else. The guard file carries roughly fifty tests, and this is the one exported front
  door not among them. Six mutants survive in the file; the sharpest is
  `src/app/sync/guard.rs:1395:21: delete !`, which **inverts** the refusal — the door would
  refuse when there is nothing to refuse and wave through when there is, so a failed sync
  uninstalls a protected package. The two `refusal_for_own_work` arms are the promise the doc
  comment above makes ("**the refusal names the package**"), and deleting either arm stops the
  message saying which package it saved, with no test noticing. **Nothing measured is wrong
  today** — the code behaves correctly; what is missing is the evidence. **The rhythm is the
  opposite of a bug fix**: the code is right today and the mutant is the artificial break, so
  each test is expected to be **green on today's tree and red against its named mutant**, and a
  test that passes on both is worthless.
  - **BUILT 2026-10-20 — and the layout in `#120` is the vehicle, not a separate piece of work.**
    The four tests are in `tests/app/sync/guard/`, one file per assertion, and the mutation job
    re-run on the file now reports **three mutants tested, three caught, none missed**:
    `delete !` in `enforce_own_work` is caught by
    `a_manager_that_could_not_answer_essentials_stops_the_unwind`, `delete match arm
    Objection::Protected{key, reason}` by `a_protected_package_stops_the_whole_unwind`, and
    `delete match arm Objection::UnverifiedEssentials{key, backend}` by the first of those.
    **The fourth survivor needed a different answer.** `Reaped::allowed` → `1` was an *equivalent*
    mutant — the getter's only reader was the text of a `debug_assert!` whose condition reads the
    private field instead — and the fix was to give it a reader that matters: the control test
    unwinds **two** packages and asserts the token says two, so the count is a fact rather than a
    coincidence of arithmetic.
  - **The move was the other half.** `src/app/sync/guard.rs` was 3,473 lines, 1,944 of them the
    test module, and over the recorded 3,500 ceiling *because of them* — the exemption row said
    so in as many words ("**plus its own test module**"). The tests are now in
    `tests/app/sync/guard/`, one file per assertion, and the file is **1,550 lines**: under the
    3,000 gate, so the `TOO_BIG_FOR_NOW` row is **retired rather than raised**, which that gate
    requires. **Eight items in the file are `pub` for the tests' sake** and say so in their own
    doc comments: a `pub fn` reached only from tests is what
    `a_pub_fn_nobody_calls_is_dead_tests` says a test double looks like, and policing those
    "would make this a gate people delete". **None of the eight mints authority** — C3 keeps
    `Reaped`'s fields private because a token must not be mintable, and these ask the guard
    rather than standing in for it. — ISSUE #115)
- [ ] #116 the mutation gate can report a mutant caught when nothing caught it. (**Filed
  2026-10-20, found while reproducing `#115`'s survivors on this host.** `cargo mutants` reports a
  mutant caught when *any* test fails, and cannot tell "failed because of the mutation" from
  "failed because the host was busy". Reproducing `#115` locally, `guard.rs:1395:21: delete !`
  came back `CAUGHT` on the strength of
  `startup_budget_tests::building_the_registry_asks_nothing_and_therefore_costs_almost_nothing`,
  which failed at `registering 56 backends took 126.3ms, over the 120.0ms budget` — a mutation
  that cannot affect registry construction, and a number this host's three concurrent `cargo`
  builds explain (the same file passes in 0.07 s unloaded). **This direction is the dangerous
  one:** a false catch is a false *negative* on the gap, the mutant is not on the survivor list,
  nobody writes the test, and a shard whose mutants are all falsely caught exits **0** — a gate
  reporting `ok` for a tree where the switch is still unpinned, which is II.23's shape. **The
  repo already knows this hazard in the neighbouring place** — `ci.yml`'s own comment says "a
  budget that tight goes red on a loaded runner — `run_timed_floor` takes the best of three for
  exactly that reason", and the budget test's header says "a gate that reddens on load is one
  people learn to ignore" — but the mutation job is the one place a reddening test is
  **load-bearing**. Three remedies are named in the issue and **none is ruled**: this is an owner
  ruling, because it changes what the gate is allowed to claim. — ISSUE #116)
- [ ] #117 the sync engine's safety checks are unpinned. (**Filed 2026-10-20 off the nightly's
  mutation output, and it is the largest of the family: 34 surviving mutants in
  `src/app/sync/mod.rs`, across eleven subjects.** The `Ok(())` family is the sharpest —
  `require_revert_path`, `require_health_commands_approved` and `rebuild_kernel_modules` are each
  "the engine checks a precondition before it touches your machine, and refuses if the check
  fails", and replacing each with `Ok(())` is "**the check always passes**", with no test
  failing. `probe_ok` → `true` **and** → `false` is the same gate asked in both directions with
  neither noticed. `prune_snapshots_after_sync` → `()` means the snapshot cleanup never runs, and
  its `Err(e)` arm deleted means the error is swallowed instead of reported.
  `execute_transaction`'s `+=` → `*=` is a counter that accumulates becoming one that multiplies
  — and the two agree perfectly at one and at zero. **Two `GuardScope` arms deleted**
  (`Rebuild`, `PurgeUndeclared`) are a *removal path with no guard scope*, which is the failure
  the enum exists to prevent arriving by a different road. — ISSUE #117)
- [ ] #118 the data lock's contention detection is unpinned. (**Filed 2026-10-20 off the nightly's
  mutation output: eight surviving mutants in `src/core/datalock.rs`, and every one of them is in
  the code that decides whether a second Shall is standing in the way.** `is_lock_contended` →
  `true` makes *every* error read as "someone else holds the lock" — a Shall that fails for any
  reason at all concludes it is blocked by another Shall and waits, or gives up, so **a bug that
  is not a lock becomes a lock**. The predicate's own body is unpinned in three ways, and the file
  asks the question in **both polarities** (`is_lock_contended(&e)` at 144 and 214,
  `!is_lock_contended(&e)` at 223), so the gate cannot tell whether either polarity is right.
  **This is one subject with four assertions, not eight bugs.** — ISSUE #118)
- [ ] #119 the transaction's worker shutdown and lock budget are unpinned. (**Filed 2026-10-20 off
  the nightly's mutation output: three surviving mutants in `src/core/transaction.rs`, shards 2, 3
  and 4.** `Transaction::quiesce_workers` → `()` is the bad one: quiescing is how the transaction
  stops its background workers before it finishes, so that a half-done run leaves no worker still
  writing to the machine while the rollback decides what to undo — and a transaction that quits
  without quiescing can leave the WAL, the registry and the machine disagreeing, which is the
  crash-window family this file exists to handle. `LockBudget::total` → `Default::default()` is a
  zero wait budget, and the CI calls this file "the transaction that decides whether a failed run
  gives a package back". — ISSUE #119)
- [ ] #120 a test layout where every src file has a home. (**Filed 2026-10-20, and it is the
  mechanism the other five need rather than a style preference.** `src/a/b/c.rs` owns
  `tests/a/b/c/`; a test that needs `cfg(test)` — because it must see a private item, or must use
  a constructor that must not ship — stays inline, and that is an **exception** with a reason
  rather than a default. **This re-derives a ruling already in the register**: `C3` (2026-09-28)
  says "unit tests use a `#[cfg(test)]` constructor that does not ship; the integration suite asks
  the guard for real", and `target-state.md`'s "**Tests ask too.**" says the same. What the repo
  has decided is *how the two sides ask*; what it does not have is a **layout** that says which
  side a test belongs on — and that is why `#115`'s function had nowhere to put a test. **It would
  retire most of the repo's own exemption list**: the five files over the 3,000-line ceiling are
  over it *because of their inline test modules* — `generic.rs` 3,940, `statement.rs` 3,881,
  `transaction.rs` 3,299, `executor.rs` 3,150, and `guard.rs` 3,473 of which **1,946 are tests**
  — and all five are rows in `TOO_BIG_FOR_NOW`, a table audited in both directions. The three
  kinds of test are separated by **what the test drives**: `tests/e2e/` (drives the product, 51
  files), `tests/review/` (reads the tree — the repo already calls that category "the review
  apparatus"), and the mirror for the rest. **Phase 1 is `guard.rs` and it is a bug fix, not a
  refactor**: the missing tests, the test module relocated, and a ceiling row retired in one
  change.
  - **PHASE 1 BUILT 2026-10-20.** `src/app/sync/guard.rs` owns `tests/app/sync/guard/`, and the
    folder holds **one file per assertion** — 56 moved and 4 written for `#115` — with the
    shared helpers in the module file beside them, because a helper in `src/` cannot be reached
    from a test crate. The file is **1,550 lines**, under the 3,000 gate, so its `TOO_BIG_FOR_NOW`
    row is **deleted**, which that gate requires of a file that stopped being too big.
  - **The scan is recursive now, because the tree is.** `every_test_file_is_in_the_suite` scanned
    the top level of `tests/` and matched `mod <name>;` lines, so a folder's contents would have
    been invisible to it — the exact failure it exists to prevent, one directory down. It now
    walks the tree, checks every `.rs` file against the module file that owns its directory, and
    checks every directory against the module file that owns its parent. **A directory with no
    Rust anywhere beneath it is data, not a module** — `tests/fixtures/` holds 47 captured command
    outputs and no source, and requiring a `mod` declaration for it would declare nothing.
  - **What the move cost, stated because it is the price of the next 186 files.** Eight items in
    `src/app/sync/guard.rs` are `pub` for the tests' sake — `MAX_LISTED`, `TOTAL_KEY`,
    `allow_the_count`, `announcement`, `too_many_changes`, `protected_names`, `enforce_total` and
    `charge_unmodelled` — and each says so in its own doc comment. **The rule this establishes
    for the sweep is: widen visibility rather than rewrite the test**, because
    `a_pub_fn_nobody_calls_is_dead_tests` says a `pub fn` reached only from tests is what a test
    double looks like, and policing those "would make this a gate people delete". The exception
    table is for the other case: a test that needs `cfg(test)` for a constructor that must not
    ship, which no amount of visibility can reach from a test crate. — ISSUE #120)
- [ ] #121 the web backend's credential-probe test fails on a coin flip, and it takes the
  mutation gate's baseline with it. (**Filed 2026-10-20, while `#115` was being built, and it is
  the *third* host-dependent test to reach the mutation gate's baseline the same day.** Two failures
  in five runs of `cargo test --lib` on an idle host, and the assertion that fails is
  `backends::web::tests::a_resource_a_system_manager_owns_is_removed_through_that_manager`: the
  recorded calls show the escalated removal went through the mock and the `sudo -n -v` probe did
  not, because **another thread had already primed the process-global `SUDO_PRIMED`**.
  **Pre-existing, and documented as unfixed in the file itself** — the comment says the reset
  "fixes the *ordering*" and that the coin flip `#88` recorded is one "this did not fix". Measured
  on the tree before this session's move: **one failure in six runs**, so the rate is the same.
  **CI has not been hit by it** — the race needs a POSIX `sudo` probe, and `escalates` is false
  on Windows, so the probe never runs there and the global is never set. **The cost is to the
  mutation gate**: `ERROR cargo test failed in an unmutated tree, so no mutants were tested`.
  **A shard that answers nothing is not a gate.** Three candidates are named in the issue and the
  first of them changes production code, so it is a ruling. — ISSUE #121)
  - **The gate's own hazard fired twice while this was being built, and both times it was the
    host's answer wearing the gate's clothes.** Reproducing the survivors locally, the mutant
    `delete !` in `enforce_own_work` came back **`CAUGHT`** on the strength of
    `startup_budget_tests` reporting `registering 56 backends took 126.3ms, over the 120.0ms
    budget` — a mutation that cannot affect registry construction, on a host with three concurrent
    `cargo` builds. Unloaded, the same file passes in 0.03s. **And later the same day the whole
    suite went red the same way** (`319.6ms` against a `120.0ms` ceiling, with a mutation job
    running), which took the mutation job's **baseline** with it: `ERROR cargo test failed in an
    unmutated tree, so no mutants were tested`. **So the gate can answer wrongly and can answer
    not at all, and both look like a red job.** That is `#116`, and it is not fixed by any of
    this.
- [x] The nightly's other red jobs were `#101` and `#107` in disguise. (**REVIEWED 2026-10-18, and
  this is why the nightly is worth reading even when its subject is elsewhere.** Every nightly from
  `38129897624` (2026-10-11) through `39109475065` (2026-10-18) is red on head `c5d0547`, and the
  red decomposes into six families — **three of which were already-filed bugs paying for themselves
  through jobs that have nothing to do with their subject**:
  - **Twelve `Mutation (…)` jobs never reach their subject at all.** They fail at `cargo mutants`'
    own baseline, an unmutated `cargo test --test suite`: `test result: FAILED. 733 passed;
    2 failed`, the two being
    `the_other_fan_outs_fan_out_too_tests::{sbom,export}_asks_the_managers_concurrently`
    (`117622290849` in `39109475065`). Twelve jobs, one cause, and the cause is `#107` — the fan-out
    gates were red for the host's reasons, fixed by `4294afc`. **The child table that diagnosed
    `#107` was printed by this job**, which is how the seven children were named at all.
    **SUPERSEDED 2026-10-20: the baseline is green, and the same jobs are red for the reason they
    exist.** Run `39392961509` prints `ok Unmutated baseline` and then reports **51 surviving
    mutants across seven of the thirteen jobs** — eight of the thirteen red, seven of them with
    survivors, and the eighth (`the batch narrowing`) dead on a cargo-mutants internal error
    (`interrupted`) rather than on a survivor. The jobs have reached their subject; the subject is
    **51 unpinned safety switches**, filed as `#115`–`#119`. **And one of the six findings is the
    gate's own** — `#116`: a mutant reported *caught* on the strength of a timing flake, which is
    the direction that hides gaps rather than reporting them.
  - **`Upstream argv drift (nightly) (windows-latest)` never failed a test either**: its log says
    `process didn't exit successfully` — a dead process, not a failed assertion, on the job that
    runs `--release`. That job's own command on this host against the current tree:
    `cargo test --release --test suite -- argv_drift_tests:: --nocapture` → `1 passed; 0 failed`
    in 155.34s. `#101` again, and the second job that was paying for it.
  - **The two native sweeps are `#112`**, including the Windows `link` `rc=139` pair now verified
    as `#101` there.
  - **The `guix` and `slackware` container legs are `#113`** — guix with nine failures *and* a
    section-floor breach naming six sections short, slackware with the one `go:` failure that the
    `arch` leg also sees.
  - **`Integration (arch)` and `Integration (void)` are the push-time legs** and are `#100`'s and
    `#109`'s subjects, not the nightly's.
  - **Three jobs on the same nights were red for reasons this session fixed**: `Advisories and
    licences`, `Build for x86_64-pc-windows-msvc` and `Build for x86_64-unknown-linux-gnu`, all
    green on `83653aa`'s push run. The Windows job went from `STATUS_ACCESS_VIOLATION` at test 480
    of 723 to the whole suite passing.
  - **Green on every one of those nights, so they are not part of the story:** both install-script
    jobs, `Integration (gentoo, nightly)`, `Integration (slackware)`'s non-`go` checks,
    `Harness mutation`, the four non-native builds and the MSRV job. `Integration (tools, nightly)`
    is `cancelled` rather than failed, which is its own thing and not a leg that ran and broke.)
- [x] #55 gentoo (emerge) leg: harness drives the ambiguous canary jq. (**BUILT — the canary is
  `htop` since 2026-08-17, because `jq` is two atoms on Gentoo and a bare `jq` is a name Portage
  itself refuses; `Dockerfile.gentoo:96` and the nightly leg is green. The issue is stale, not
  wrong: it was true when filed. **CLOSED 2026-10-18, and on four nightlies rather than on this
  entry** — `38703692718`, `38871047055`, `39014498876` and `39109475065` all report
  `Integration (gentoo, nightly) success`. This entry claimed the leg was green and a claim is not
  the instrument; the four runs are.)
- [x] #90 fan-out overlap floor fails a skewed host. (**BUILT, and the fix is a skip rather than a
  number, which is the whole argument.** The ratio is `sum(child)/wall`, and the smallest wall any
  scheduler can reach is the *slowest* child — every other child hides inside it. So the ceiling is
  `summed/slowest`, and when one child holds most of the summed time that approaches 1.0: measured
  on the five-manager host, `emacs --batch` alone took 0.36s of 0.46s summed, giving a ceiling of
  **1.28x against a floor of 1.25x** while every run started all five children at once and finished
  in one wave. No scheduler on that host can score what the floor asks for.
  - **The issue offered three routes and named this one the only safe one.** Lowering the floor to
    the ceiling (`min(floor, ceiling*0.9)`) is the same mistake twice — a genuinely *serial* run
    scores ~1.0 there and would pass. Raising `min_children` silences legitimate small CI hosts the
    field's own comment wants included. Skipping says the measurement does not exist, and the
    load-independent half of the pair still runs.
  - **`waves` is what covers the gap**, and this is why the skip costs no detection: one wave is a
    perfect fan-out and one wave per child is a serial loop *whatever the child durations are*. So
    collapse stays detectable on exactly the host where the ratio gave up. Asserted in both
    directions — the skewed one-wave run must not be reported, and the same children run five waves
    deep must still be caught.
  - **One predicate, two callers, unchanged discipline.** `shape_measurement_gap` replaced
    `is_measurable` as the single answer to "is this host measurable", and returns the *sentence*
    rather than a bool, so the rule and the reporter cannot disagree — which is the failure mode
    the field's own doc calls the worst kind this repo keeps finding. `overlap_headroom` (1.25) is
    the new key, and it is a multiple of the floor rather than a second absolute constant. — ISSUE
    #90)
- [x] #107 `sbom` and `export` were filed as asking their managers one at a time. (**BUILT — and
  the title was refuted by the gate's own evidence, which is the larger half of what this entry
  records.** Filed out of #90, which was closed on a DIFFERENT signature, and the distinction is
  the whole point of filing it separately.
  Both fail on a 7-manager host at **6 waves over 7 children** — close to one-at-a-time. #90's
  evidence was a *one-wave* run on a skewed host, where the low ratio was the ratio's ceiling
  (`summed/slowest`) rather than a scheduler. `Shape::wave_ceiling(7)` is 4, so the wave assertion
  fails too, and #90's fix deliberately withholds the skew exemption above one wave so a serial run
  is still judged. **They are judged correctly and they fail correctly** — checked before filing,
  because re-reading them as "#90 again" and re-closing them would have been easy and wrong.
  - **Not established: where the serialisation is.** `resolve_managed` is a wrapper over
    `export::managed_pkgs`, which is `stream::iter(..).buffered(max_parallel)` with `max_parallel`
    = core count; `info`'s singleflight is keyed **per backend**, so seven backends take seven
    mutexes; the executor has no spawn semaphore. Every obvious candidate is ruled out by reading.
  - **And the prior question is whether the fan-out is what is being measured at all.** On a machine
    with an empty registry `managed_pkgs` fans out over nothing, so the seven children came from
    elsewhere — an `App::new` warm-up being the obvious candidate. If so this is a *third* instance
    of the family #89 and #100 both turned out to be: **a gate reading the host instead of the
    state it meant to set up.**
  - **Not reproducible here** — this host has ONE manager, below the floor's own minimum of 4, so
    the test skips and proves nothing locally.
  - **WHICH CHILDREN ARE THE SEVEN — ANSWERED, and it was answering itself all along.**
    `timing::report` has always printed a per-child table under the summary line (`at`, `took`,
    `command`, one row per label), and the fan-out gate captured that whole string, kept the single
    line beginning `Timings:`, and **threw the table away**. So every failing CI run of this gate
    printed the evidence and discarded it in the same breath, which is exactly why the
    serialisation could be ruled out in every candidate function and still never located.
    `timings()` now returns the summary line to parse **and** the report to quote, and both failure
    messages print the table: a failing run names the managers, when each started, and how long
    each took. The next run that goes red is the answer rather than another question.
    `render` is split out of `report` so that "the report names every child" is a unit test instead
    of something a person has to notice in a log. The summary line is byte-identical, because the
    gate parses it — a report that fixed the diagnosis by breaking its own parser would trade one
    unreadable failure for another. Mutation watched: dropping `row.label` reds the test. — ISSUE
    #107)
  - **THE SEVEN CHILDREN ARE NOW NAMED, and they say the gate was measuring the wrong unit.**
    Run 37232355019 was the first to print the table, and it names them: `npm list` 2.02s,
    `conda list` 1.60s, `dotnet tool` 0.69s, `pipx environment` 0.47s, `pipx list` 0.40s,
    `npm prefix` 0.17s, `dpkg-query -W` 0.04s (`sbom` the same seven with different durations).
    - **Seven child commands, FIVE managers.** npm is asked twice (`list`, `prefix`) and pipx twice
      (`list`, `environment`), so "asked 7 managers" named a number that is neither a manager count
      nor a command count — and `children` is what goes to `is_measurable` and `wave_ceiling`,
      **both calibrated on managers**. The gate was measuring in commands and reporting in
      managers. `render` now prints the distinct count when it differs, and `distinct_programs`
      counts **programs** rather than managers on purpose: a manager is a Shall-side concept and a
      span records a process, so calling it `managers` repeats the overclaim one level down.
    - **"That is asking them one at a time" was refuted by the table printed under it.** `npm list`
      and `dotnet tool` both start at 1.65s; `pipx list` overlaps `npm prefix` and `pipx
      environment`. It runs about **two wide**. The assertion is right to be red; the prose was
      wrong about why, and both messages now say to read the table before concluding serial.
    - **So `managed_pkgs` is not what these two commands use** — these seven children arrive in
      pairs per backend, which is not the shape of a `buffered(n)` over packages.
    - **The seven children decompose: FIVE listings plus TWO property probes.** `npm list` +
      `npm prefix` and `pipx list` + `pipx environment` are one listing and one
      `PropertyProbeDef` probe each, so the second call per backend is a probe running *after* its
      listing. Real, and not yet the whole story.
    - **The hypothesis that SHOULD have held, tested, and did not.** `max_parallel` bounds
      *processes* and defaults to `available_parallelism()` — **2 on that runner** — while
      `managed_pkgs` fans out over *packages*, so a package whose backend is already being asked
      holds a slot waiting on the per-backend singleflight instead of asking anything. The table
      fits: `conda list` alone for 3.1s on a two-slot runtime. I implemented it (group by backend,
      fan out over backends, restore document order by index); it is **correct** — all twelve
      packages returned, order preserved — and **not a win**: 815ms vs 318ms for one package each,
      a **2.6x** ratio where grouping predicts ~1.0x. **Reverted.**
    - **CORRECTED: THE MEMO WAS NEVER BROKEN. THE FIXTURE WAS.** Every fan-out measurement in
      `a_fan_out_of_managers_is_asked_concurrently_tests` was taken on the **error path**. The
      mock registered `AptParser` and answered `mock0 list` with `mock0-pkg 1.0`; `read_row`
      requires a status word dpkg can emit, so the row was **unreadable** — and unreadable is an
      **error**, not an empty machine. `once` deliberately **does not cache a failure**, so every
      fetch errored, the memo never filled, and 4 managers × 3 packages produced **12** child
      commands where 4 were intended. Correct row: `installed  mock0-pkg  1.0`.
      **Consequences, stated so neither is repeated:** (a) `installed_listing` is fine — 4 listings
      for 12 lookups, asserted by a count test with two mutations watched; (b) **the grouping
      change reverted above is still unproven, not refuted** — its 2.6x was measured on the error
      path and says nothing either way; (c) the CI observation is untouched and still stands.
    - **The timing assertions in this file are host-sensitive, with receipts, so they ask a
      counter rather than a clock.** `seven_managers_are_asked_at_once_and_not_one_after_another`
      failed at 875ms and then at 1.6s against an 840ms serial baseline purely because `ksav` was
      running a `--release` build here (load 33), and passes at 0.28s idle before and after the
      change. **Both are now peak-concurrency counts**, wrapped in the test as an
      `ExecutionLayer` — adding the counter to `MockExecutor` was refused by
      `a_module_is_a_subject_not_a_pile_tests` at 3223 lines against a 3150 ceiling, correctly. A
      clock measures the host as well as the code.
    - **CLOSED 2026-10-18, on the one reading that cannot be fooled. The two gates were red for two
      separate instrument reasons and neither was a scheduler.**
      - **The ratio reads the host.** Windows: 7 children summing to 5.23s inside 4.25s — **1.2x**
        against a floor of 1.25x. Linux: 1.89s inside 1.68s — **1.1x**. Both runs started two
        children at once (the tables above say so), so the low number was process-startup cost and
        not sequencing.
      - **`max_parallel` is the core count, which is 2 on both runners**, so five declared managers
        (seven commands, two of them a listing *and* its probe) cannot reach 1.25x or
        `wave_ceiling(children)` waves however well they are scheduled. Linux's "6 waves over 7
        children" was arithmetic about a two-slot budget. `#90`'s entry established the ratio's
        ceiling (`summed/slowest`); this is the other bound on the same number, and neither is a
        fact about the fan-out.
      - **`core::timing` now reports `width`** — how many children shared the busiest instant —
        computed in the same `schedule()` sweep as the wave count, so a hand-off cannot be counted
        as overlap by one reading and as a wave by the other. It is appended to the same `Timings:`
        line. A `for` loop awaiting each child is exactly 1 wide on every host, which is the defect
        the title named, and that is what
        `tests/the_other_fan_outs_fan_out_too_tests.rs::assert_not_serial` asserts now: one wide is
        a collapse, and anything wider was not serial whatever its ratio.
      - **`latency::report_shape` uses the same reading to stop warning a user**, on any host with
        two slots, that an ordinary `sbom` "did not overlap the managers" — it records the run as
        unmeasured at `info` instead, because the ratio and the wave ceiling are bounded by the
        run's own budget rather than by the scheduler. **What that gives up is stated in the code
        and in the issue below**: a fan-out using two slots where it had sixteen is no longer
        reported by any number in the tree.
      - **The ratio and the wave ceiling stay enforced where they were calibrated** — over `list`
        in `tests/latency_budget_tests.rs`. Every gate that reads them is still asked, none was
        deleted or weakened, and the two in this entry were re-pointed at the reading that means
        the same thing on a 2-core runner as on a 16-core one. — ISSUE #107)
    - **CONTROL ADDED: the reported seven-child shape is now exercised without a clock.** A
      deterministic five-backend fixture gives two backends one `PropertyProbe`, so
      `managed_pkgs` must produce five listings plus two probes. The regression asserts exactly
      seven calls, preserves the state/document order, and reaches but does not exceed a
      `max_parallel = 2` peak. It passes with the corrected successful-listing fixture. This
      covers the observed `npm list`/`npm prefix` and `pipx list`/`pipx environment` shape and
      proves the probe is not a seventh manager or a bypass of the process cap.
- [ ] #111 the manager fan-out is bounded by the core count, and no instrument reports the degree
  it lost. (**Filed 2026-10-18 from the two measurements above, as a budget question rather than a
  defect.** `max_parallel` defaults to `available_parallelism()`, which is right for work that ends
  in a CPU and wrong for a manager listing — a spawn plus a wait — which is the same argument the
  config already accepts for `network_parallel` on sockets. **Nothing measured is wrong today**
  (both tables show two children in flight at once, so neither command is serial) and **no
  benchmark exists for the win**, which the issue says in its own evidence section: the first step
  is to measure on a many-core host, and on a two-core host with the budget raised. Three candidate
  routes, the first being one config field defaulted from `max_parallel` and grouped by
  `app::stale_lock::lock_key` — the function that already knows `pacman` and `yay` are two programs
  over one database, which is the failure a wider fan-out could introduce. **An owner ruling, so
  filed rather than landed**, and route 3 is to measure it and close it as not worth doing.)
- [x] #105 the crash window between a per-operation WAL write and the once-per-run registry write.
  (**ROUTE (2) WAS ALREADY BUILT — the issue's own "better next move" — so what was left was its
  residual, and the residual has one cause: the evidence expires.** `heal` calls
  `reconcile_ownership` first and *unconditionally*, and `reconcile_ownership` reads
  `completed_installs`, so an orphan **is** repaired on the next sync. But
  `cleanup_expired_logs(7)` drops a `Completed` entry after a week, while the claim it witnesses
  does not expire: a machine left alone for a week loses the only evidence that would have repaired
  its orphan, and falls back to a listing that correctly reports an unpacked-but-not-configured
  package as *not installed* — which is the lister being right. Closed by #94's receipt below.
  **Route (1)** (ownership written per operation / same commit point as the WAL entry, then the
  reader goes) is the better end state and remains the owner's; it is an architecture change to the
  transaction, not a fix, and deleting the reader before either route is the one unavailable option.
  **BUILT — the resolving test, 2026-10-09.** The entry above said the residual was closed by the
  receipt and nothing said so in code, so it was a claim with no gate: remove `receipted_installs`
  from the union in `reconcile_ownership` and nothing fails, because no test aged a journal entry
  past the purge rule. `an_orphan_whose_wal_entry_has_expired_is_taken_back_on_the_receipt` does
  exactly that — one `Completed` install aged past `cleanup_expired_logs(7)` by hand, a manager whose
  listing holds nothing, and the orphan still taken back. **Mutation watched:** with the `chain`
  taken out, 17 of the file's 18 tests pass and this one fails with *"the only evidence left was the
  receipt, and it was not read"*. Two controls inside it so it cannot pass for the wrong reason:
  the purge must report having dropped something, and the journal must be empty of the entry
  afterwards — otherwise the test is measuring `completed_installs` again, which is the reader this
  residual is about not depending on. — ISSUE #105)
- [x] #106 Windows MSVC. (**BUILT. The one real failure was #101 arriving through the front
  door, and the reason it read as a defect of its own is the second half of this entry.**
  `check_reports_a_file_it_would_place` failed because `shall check` never printed a byte: the
  fixture's `link:` resolves its `@target=` through `Account::current()`, which died with
  `STATUS_ACCESS_VIOLATION`, and the control's `row` was empty because the process was. With that
  fixed, the same fixture on Windows prints
  `->  drift  0 to install, 0 to remove, 1 to place, 0 to undo` — the control holds, and the row
  keeps the resource through the skip
  (`… ; 1 declaration(s) this machine cannot act on, so they will not be installed: apt:tree …`).
  The two `FAILED` lines in the original issue were one crash in flight, and
  `a_ledger_without_a_floor_refuses_to_audit` was never a failure at all: it is `#[should_panic]`,
  so `--nocapture` printing its panic is *success*. The removal-token gate was fixed in
  `c11498d`.
  - **The defect that was really in this file is that the other three tests passed the whole
    time.** `add_a_declaration_this_host_cannot_serve` returns `Option`, and each test read `None`
    as "nothing to measure here" and returned early — but a `shall` that dies prints *nothing*,
    nothing contains the `is not on this machine` warning, so every candidate manager was
    "rejected for being installed", the helper returned `None`, and a file whose entire purpose is
    to prove a fact does not vanish reported a pass over a machine where the fact was never
    checked. **That is #89's shape one level up: a control that never held.** It is now a failure
    when `plan` prints nothing at all, naming the candidate and the directory, and the manager it
    settled on is printed on the way past — a gate that refuses to run beats one that passes
    vacuously.
  - **With the crash gone the suite ran to completion, and that is what the crash had been
    costing: eight more gates, all of them red on the platform they had never run on.** 723 passed
    and 8 failed, and not one of the eight was a defect in what it was measuring — each was the
    gate's own portability, which is why grepping the tree for `#[cfg(windows)]` would not have
    found them:
    - **Four `feature_logic_tests` ledger reads looked for a spelling nothing wrote.** A row holds
      the *declaration's* subject (`decl()` normalises to `/`), and the lookup used
      `Path::display()`, which is `C:\Users\…` on Windows. The directory was recorded correctly and
      the test searched for a path in a spelling nothing had used.
    - **`prompt_guard_tests` parsed a caller out of a string with `split(':')`**, which is the drive
      letter on Windows — so every caller was filed under `C` and `sync.rs` looked like a file that
      never asks the outside-home question. The answer is a `PathBuf` and a line, not a smarter
      parse.
    - **`the_probe_argv…` asserted npm's POSIX layout on both platforms**, while the row itself
      carries a `windows_template` because `lib/node_modules` is a POSIX fact. The subject is the
      *per-package* half of a shared answer, so the expectation is per-platform now and the
      property is unchanged.
    - **`every_shell_script_the_repo_runs_has_lf_endings` was right and the tree was wrong.**
      `scripts/section-floor.txt` is bind-mounted and read in-container; `scripts/lifecycle-floor.txt`
      beside it got an `eol=lf` entry and this one did not, so a `core.autocrlf=true` clone wrote it
      CRLF. `.gitattributes` says so now — the gate is the reason that entry exists.
    - **`a_directory_mode_edit_is_reported_and_then_converges` asked a Unix question on Windows.**
      `@mode` is refused there by name, by design, so convergence is not available to assert. The
      Unix test keeps the convergence and the Windows run gets the refusal — *by name*, and *before*
      anything is created — so neither platform's run measures nothing.
  - **And one real product bug behind them, which is the one that mattered.** `extras::in_effect`'s
    metadata read was `#[cfg(unix)]` because the only thing that read it was the mode comparison —
    so on Windows a declared `dir:` reported **`the machine matches your files` when the path was a
    regular file, and when it was not there at all**. `install` refuses the first by name and creates
    the second: the two halves of one declaration disagreed about one directory, and the reader was
    the half that was wrong. Fixed, and proven through the CLI rather than through unit tests —
    `drift … 1 to place` for the missing one and for the file, `ok … matches` for the directory.
  - **Two clippy lints fired here and nowhere else**, both in `account.rs`'s `#[cfg(windows)]` half
    (`needless_return`, `manual_is_multiple_of`). `-D warnings` is CI's gate and CI lints Linux, so
    the half of a file that only compiles on Windows is the half no gate lints — the same sentence
    as everything above, said about the linter. — ISSUE #106)
- [x] Suite-isolation bugs (Rust suite green only on NOPASSWD-sudo/adoptable hosts). **All three
  closed**: #88 (mock-layer tests probe real sudo — the probe now goes through
  `layer.execute("sudo", &["-n","-v"], &{})`), #89 (see its own entry), #90 (see below).
  **What the family was:** a gate that reads the host when it meant to read a fixture. #88 and #89
  both failed *for having nothing to measure* and both are now built rather than found — and #89's
  finding generalises: a count taken from the wrong file is the same mistake as a control whose
  fixture never held.
- [x] #38 eopkg RETIRED honesty. (**BUILT, and it is a third stamp rather than a flavour of the
  existing one.** `UNVERIFIED` says *nobody has looked* and leaves open that somebody could; eopkg
  cannot be paid off, because Solus publishes no image on any public registry and the project has
  no maintainer. `FixtureDef` grows `is_unverified`/`is_retired` beside `is_verified`, and **a
  retired row is not verified** — the two stamps answer different questions and a row must not be
  both. `proving.rs` asks retirement first and with a reason of its own, because *never captured* is
  a debt this repository could discharge by running the right image and *retired* is a fact about
  the world that no image can change; reporting both alike makes a dead project look like
  outstanding work.
  - **The roster keeps its `[READY]` tag, deliberately** — that tag is what
    `shall check health | grep '^\[READY\]' | awk '{print $2}'` enumerates, and a retired row is
    still registered. Dropping the tag would remove it from the one listing that names every
    backend this build knows, which is a worse lie than the suffix.
  - The harness reason said *"there is no Solus image in this matrix"*, which is the exact confusion
    — a hole in **this** repository's coverage over a project with no image anywhere.
  - `a_retired_row_reads_retired_and_is_never_driven` also asserts the harness has **no lifecycle**
    for it. That is the check the stamp exists to enable, and the mutation (adding `eopkg` to
    `DRIVEN`) reds it.
- [ ] #41 prose tax distill. (**#36 is CLOSED** — its second documentation layer was a citation
  with no venue, and `c5d0547` shipped the fix on 2026-10-09 — so this item is #41 alone. The half
  below is what is deliberately not started.)
  - **#41 IS DELIBERATELY NOT STARTED, and the reason is that it is not a mechanical edit.** The
    issue asks for `decisions.md` (641 KB) and `why.md` (521 KB) to be distilled to one line per
    entry. Every one of those entries is cited by line number from elsewhere in this repository —
    `docs/spec/plan.md`, the `why_entries_are_attached_to_something` gate, and this file — so the
    distillation does not leave the prose smaller, it leaves **several hundred citations pointing at
    nothing**, and the gate that checks citations is what would catch it. Distilling in the right
    order means: index the citations first, move the durable rule per entry, then delete. That is
    three passes over 1.2 MB and it wants its own branch, not a commit between two unrelated fixes.
  - [x] #36 the ruling vocabulary cited from code had no venue and no resolving test. (**BUILT, and
    the twelve it found the day it was written had been lying for a year.**) The issue asked for a
    new `docs/rulings.md`; **it was not built, because `docs/SPEC.md` already names the venue** —
    "`docs/spec/` is the specification", `decisions.md` the register and `bugs.md` the S-series — and
    `id_namespaces_do_not_collide_tests` already treats that whole directory as the register and its
    annotations. A new file would be a *second* venue, which is the disease this repo keeps finding.
    `every_ruling_cited_in_code_resolves_to_one` is the twin of `every_citation_of_a_why_entry_
    resolves_to_one`, over the register's reserved prefixes (`D W K N T U`) plus `Q` and `S`, and
    `V` is deliberately left to the gate that already resolves it — a second oracle for one citation
    is a contradiction waiting to happen.
    - **Two families, and both were the document's fault rather than the code's.** Ten citations
      were BUILDER's own work orders: `docs/BUILDER.md` renumbered `W1`–`W43` to `B*` when
      `id_namespaces_do_not_collide_tests` shipped, and the fifteen places citing them did not
      follow — so `W21`, `W25`, `W35`, `W36`, `W37`, `W38`, `W41` and `W43` had been reading as
      *"stop and ask the owner"* ever since, which is the one reading a stale citation should never
      keep. All sixteen renumbered (`W38` was in two files; the twelfth surfaced only after the
      exemption was made per-corpus). The other two were **`S22` and `S23`, real bugs that were
      fixed and never given a row in `bugs.md`** — `pixi`'s empty-result banner read as a package
      called `No`, and `nimble`'s format legend read as two — so the three places naming the
      failure class cited nothing. Both rows now exist, reconstructed from the two commits that
      fixed them and marked as reconstructions, because a row written after the fact that does not
      say so is indistinguishable from one that was always there.
    - **The exemption is per-corpus, and that was a bug in the first draft.** Keyed by ID alone it
      excused `W43` *everywhere*, including a stale citation in `dry_run_every_verb_tests.rs` —
      a permission granted to one file and silently extended to the tree. **Mutation watched, and
      it is the better of the two:** re-pointing an exempt citation reds both halves at once, the
      exemption naming itself stale and the new dangling name in the same run.
    - Two controls that stop the scan reading vacuously: the tree must have >100 Rust files and the
      venue >100 definitions, `U27` (a `## U27` heading) and `S87` (a `| **S87** |` row) must be
      found and `U999` must not. **`W32Time` — the Windows service — was read as a citation of a
      work order called `W32`,** and the first version's own example text (`W32`, `U999`) was
      cited by the file that quotes examples; both are why the scan skips self and requires a
      boundary *behind* the digits as well as in front. — ISSUE #36)
- [x] #95 `V.186` and `Q55` said `completed_installs` was deleted; the code still had it, reading
  it in production inside `reconcile_ownership` (`src/app/sync/mod.rs:1024`). (**BUILT as a spec
  correction, and the fork was never needed — the docs were wrong in two places, not one.** The
  entry sat here reading *"needs the owner's fork: the code loses the reader, or the docs gain it
  and say what it may decide"*, long after `docs/spec/plan.md` had already ruled it (Tier 29,
  item 44). **This file was the stale one**, which is the drift in the opposite direction from the
  one the issue filed, and worth recording: the issue was fixed and the ledger kept asking.
  - `why.md` and `decisions.md` both said the function was deleted. Both now say it was not, and
    say what it may decide: `reconcile_ownership` only ever **claims a package the manifest already
    declares**, on the evidence that Shall's own journal recorded installing it. The reader is
    never an adoption source — the manifest stays the whole of ownership, which is what Q55 ruled
    — so it is evidence of Shall's own action rather than a second record of one relationship.
  - The second falsehood, which the issue did not have: `why.md` also claimed the crash orphan
    *"is covered by the manifest as a special case… nothing is left that only the log can see"*,
    while describing as deleted the very reader that saw it. That sentence is what made the loss of
    the reader look survivable.
  - **The ruling deleted nothing in the code, and that was the right side to leave alone** — the
    reader decides whether a declared orphan is reclaimed, and a crash-stranded package is
    unpacked but not configured, which every listing correctly reports as *not installed*. Removing
    the reader would have restored that defect rather than fixed a documentation error.)

## Phase 4 — Feature gaps (Info, after core is safe)
- [ ] Adopt discovery: #59 dotfiles, #60 schedules, #61 repos/PPAs, #62 shell, #63 settings, #64 services, #65 firewall.
- [ ] Export: #66 from-scan, #67 from-declaration.
- [ ] #72 nixos activation gap (CI leg), #73 search-index noun, #74 crown-jewel invariant.
- [x] #94 Shall records what it installs, and asks before installing a bare tool name (owner ruling
  2026-09-28; both halves built). **1. BUILT** (and it closes #105's residual, which is the same defect
  wearing a different hat). `core::receipt` — `receipts.jsonl` beside `journal.jsonl` under the
  data root, one line per install, **never expires**, `[receipts] enabled` (default on).
  - **Written from `Journal::record_success`, which is the one place that already sees every install
    completion** — `journalled`'s nine callers, the engine, `apply`'s execs, `heal`. Not a new
    registration step, and the file's path is derived from the WAL's own directory exactly once
    (the mistake `Journal::at`'s own doc records: every `cargo test` appending to the developer's
    real journal).
  - **A record, never an ownership source.** `reconcile_ownership` unions it with
    `completed_installs`, and it only ever claims a package the manifest **already declares**, on
    evidence of Shall's own action. II.56/V.186/Q55 are untouched.
  - **Idempotent, and the claim is *whether* not *when*** — a package installed, removed and
    installed again is one receipt, so the file cannot become a count of attempts. The timestamp is
    carried anyway so the file could be aged later if that is ever wanted.
  - **Turning it off drops the reader as well as the writer**, or the toggle would govern only what
    is recorded and not what is believed.
  - **Why this is #105's residual and not a new feature:** `cleanup_expired_logs(7)` expires the
    evidence for a claim that does not expire.
  - **2. BUILT.** Bare-origin provenance is carried on the resolved install spec, so it is not
     inferred from `BareLock`. `sync` and `apply` ask immediately before execution, using the
     shared prompt convention: `--yes` answers yes, an interactive no leaves the declaration in
     place and installs nothing, and a noninteractive run refuses with `--yes`/`--dry-run` advice.
     `plan`, `check`, `status`, and dry-run paths never ask. Saved plans use schema 3 so an older
     artifact without the provenance cannot bypass the question. — ISSUE #94
- [x] #89 guard-reachability control builds a vacuous fixture. (**BUILT — and the state the control
  needs turned out to be reachable, but not through any of the three shapes the fixture had tried.**
  `plan` wrote 2 removals with no guard refusal because nothing protected them, and the missing
  piece was an *objection*, not a declaration.
  - **A count ceiling is the only objection a fixture can manufacture.** The
    `would be removed (...)` lines come from OS-essential protection, and nothing a fixture sets
    reaches it; the fixture now writes `[guard] max_removals = 1`, so two removals are over it.
  - **`[guard] protected_packages` looks available and is not.** Protecting every adopted name makes
    the *planner* decline each removal before the guard is consulted (`planner.rs`'s
    `Declined::Protected`, which matches config rules and deliberately not OS essentials), so `plan`
    writes an empty plan and refuses nothing. Measured with `protected_packages = ["*"]`: *"2
    package(s) installed and declared nowhere that `sync` will not remove"*, and no refusal. That is
    why the control accepts a refusal **or** a protected-package line rather than the second alone.
  - **A fixed `1` rather than one below what `adopt` found**, because such a number must count
    package declarations while ignoring the `service:`/`link:` rows in the same files; an over-count
    raises the ceiling above the removals and the guard goes quiet again.
  - **The "is there anything here" count moved to where the question is asked.** It was
    `s.lines().count()` over the module files, which counted `init`'s scaffolded starter and then
    the ~35 comment lines every adoption manifest carries — so a host that adopted one package and a
    host that adopted none both read as "the fixture built". `removals_planned` reads it out of the
    plan `plan` wrote, which is also the only place that knows which declarations became removals.
  - **Which makes it a control rather than a gate that fails for having nothing to say:** no
    objection over fewer than two removals skips loudly (`max_removals = 1` refuses two and cannot
    refuse one); no objection over two or more **fails**, naming the count. That branch is the
    mutation watched — with `preview_refusals` removed from `verbs/plan.rs` the control fails, and
    the old control would have skipped it.
  - The protected-names test skips **loudly** here with its reason and still measures on the tools
    image, where the adopted set really does include OS essentials. — ISSUE #89)
    finished. — ISSUE #89)
- [x] #88 hermetic mock-layer tests probe the host's real sudo. (**BUILT, in the second of three
  proposed shapes, because the first two treat the symptom.** `ensure_sudo_credentials` spawned
  `Command::new("sudo").args(["-n","-v"])` with its streams nulled, which made it the one command
  `with_layer` could not see. It now calls `layer.execute("sudo", &["-n","-v"], &{})` and reads
  `.status`, so the probe and the escalated command arrive at the same seam. **The credential was
  never the thing under test; the argv was** — with the probe around the layer, a host without
  NOPASSWD sudo refused *before* the mock saw the `sudo dpkg -r fd` it had stubbed. **Strictly more
  capable than skipping the probe**, because a layer can now *answer* it, so the refusal path is
  reachable in a test at all; skipping makes the three tests green and leaves that path untestable.
  The issue's proposed assertion ("the probe never appears in `get_calls()`") is the skip shape's, so
  the test asserts the opposite and would fail under a regression to either shape. All seven
  `backends::web` tests pass here with no `PATH` shim, and those three have failed in every local run
  of this session. **And the reset that did not reset, found by the assertion above:**
  `SUDO_PRIMED` is process-global, so the probe only runs in whichever escalating test reaches it
  first — the first run of the new assertion passed and the second failed on nothing but libtest's
  thread order. `forget_sudo_refusal()` cleared the refusal and not the warm flag; it now clears
  both, and the test calls it before asserting. **Not taken initially because `src/core/executor.rs`
  sits at exactly its 3,150-line ceiling** — routing the probe through the existing
  `ExecutionLayer::execute` is *smaller than the code it replaces*, which is why the trait-method
  route (33 lines, 32 over the ceiling) was dropped, and why these two lines came out of comments
  that said the same thing more briefly. — ISSUE #88)
- [x] #103 four duplicated comment blocks, two of them merge artifacts. (**BUILT — and the sweep
  found a fifth the hand-scan had missed.** Two were unambiguous internal repeats, now gone:
  `backends/generic.rs` carried its three-line note about rpm's `%{name}` twice on consecutive lines,
  and `model/resolve.rs` carried a **six-line `///` doc paragraph twice inside one doc comment** — so
  `cargo doc` published the set-math rule as a paragraph written twice. The gate
  (`tests/no_comment_says_the_same_thing_twice_tests`) then found **`parsers/ecosystem.rs`**, where
  two *near*-copies of a pixi doc had been merged with the opening line reworded and the other five
  lines duplicated; the two halves each carried something, so they are merged into one paragraph
  rather than one deleted. **The rule is deliberately narrow** — one comment run may not contain a
  copy of itself — because a comment repeated in *another item* is not a merge: the two `--json` flags
  in `cli/args.rs` share a note because rustdoc renders each field's docs separately, and the three
  registrar copies in `registry/os_native.rs` are a smell whose fix is a judgement about where the
  constraint belongs, so both sites are **named in the gate** rather than left as an omission. One
  predicate, its control in both directions, and the mutation watched: a two-line paragraph planted
  twice in a parser is caught with its file and line. **And the gate's first version printed
  "lines 19+4" with no file to open, which is how the file-line got carried.** — ISSUE #103)
- [x] #95 V.186/Q55 say `completed_installs` was deleted; the code still has it. (**BUILT — and the
  docs were wrong in **two** places, not one.** `why.md` said the function "was deleted rather than
  kept as a second source" and `decisions.md` said the same; both were false, and `why.md` made a
  second false claim on the way — that "the crash orphan the log was introduced for is covered by
  the manifest as a special case rather than needing its own mechanism". **It is not covered:** `heal`
  filters on `InProgress | Abandoned` (`journal.rs:514`) and this entry is `Completed`, so nothing but
  the surviving reader can see it. Both passages now say what is true: ownership is the manifest
  alone, and the reader that remains is not an ownership source — it may only *claim a package the
  manifest already declares*, on Shall's own record, which is the crash window between a per-operation
  WAL write and a once-per-run registry write. **The owner's question — "the manifest is king, no?" —
  is right about ownership and does not reach this**: what a crash loses is the manifest's own write,
  so deleting the reader would not make the manifest more authoritative, it would make a class of
  package permanently unremovable. The gap itself is filed as **#105** with both closure routes in
  safe order, because closing the window and removing the reader are two decisions and only the first
  is available. — ISSUE #95)
- [ ] Interop/process: #42, #43, #44. Process: #17–#21 bundles (split per sub-item when working).

## Routing rule for new issues
Any AI opening an issue here MUST insert it into the phase above it belongs in (foundations → safety → correctness → features). Primitives (#69-style) and safety bypasses go in Phase 1–2 even if filed later. See AI_ISSUE_ROUTING.md.

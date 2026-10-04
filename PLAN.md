# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58, #69, #70, #71, #34, #37, #75, #76, #77, #78, #35, #99, #97, #102, #98, #91, #86, #85, #81, #83, #84, #79, #82, #87, #95, #103, #88.

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
- [x] #40 two oracles that cannot fail. (Medium — both shipped; the `unstubbed` half has a
  measured follow-up of 66 tests, recorded in `docs/spec/plan.md` Tier 14)
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
- [ ] #101 CI: both MSVC test binaries die with `0xc0000005 STATUS_ACCESS_VIOLATION` mid-run, and
  the log cannot say which test crashed. (High — **this is what the fourteen compile errors were
  hiding**: the crate compiles and then the process dies, so `cargo check` cannot answer it. The
  job runs without `--nocapture`, so the crash discards its own evidence — **one**
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
  - **STILL NOT FIXED, and deliberately not pretended to be.** No Windows and no Wine here, so it
    cannot be reproduced or bisected. What is new is a *name* on both halves, which turns "cannot be
    reproduced" into "run these two tests". The next step is `cargo test --test suite
    argv_drift_tests -- --nocapture` and `cargo test --lib app::apply::dotfiles -- --nocapture`
    on a Windows runner: both dying alone means two independent bugs, both passing alone means an
    interaction with a concurrent test.
  - **#106's one remaining failure is blocked on this**, since a `FAILED` line flushed before an
    access violation is not a count of what is broken.)
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
- [ ] #25 lifecycle jobs on distro containers, #53 nimble Windows, #54 dirty-host fixtures.
- [x] #55 gentoo (emerge) leg: harness drives the ambiguous canary jq. (**BUILT — the canary is
  `htop` since 2026-08-17, because `jq` is two atoms on Gentoo and a bare `jq` is a name Portage
  itself refuses; `Dockerfile.gentoo:96` and the nightly leg is green. The issue is stale, not
  wrong: it was true when filed)
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
- [ ] #107 `sbom` and `export` ask their managers one at a time. (**Filed out of #90, which was
  closed on a DIFFERENT signature, and the distinction is the whole point of filing it separately.**
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
    - **Next step, now specific rather than a question:** attribute each `Span` to the backend
      that spawned it, so `is_measurable`/`wave_ceiling` count managers and the overlap ratio keeps
      counting commands (a ratio of sums, correct as it stands); then account for the second call
      per backend — `npm prefix` and `pipx environment` are `PropertyProbeDef` probes — and say
      whether it runs behind the fan-out or inside it. That is what makes a five-way fan-out run
      two wide over six waves.
- [ ] #105 the crash window between a per-operation WAL write and the once-per-run registry write.
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
- [ ] #106 Windows MSVC. (**DOWN TO ONE OBSERVED FAILURE, and the count in the issue was not
  reliable — the crash got there first.** `a_ledger_without_a_floor_refuses_to_audit` is a
  `#[should_panic]` test, so `--nocapture` printing its panic is *success*, not failure; it never
  appears in any `FAILED` list. The removal-token gate is fixed. One real failure remains,
  `check_reports_a_file_it_would_place`, whose control printed `row: ` and nothing else — so the
  control now prints all of `check`'s output and its exit status. **BLOCKED ON #101**: the suite
  ran **480 of 723** tests before dying with `0xc0000005`, so every test after the crash point —
  including all four ledger tests — never ran at all, and a `FAILED` line flushed before the crash
  is not a count of what is broken. Fix the crash before reading this list again. — ISSUE #106)
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
- [ ] #41 prose tax distill, #36 doc-comment layer.
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
- [ ] #94 Shall records what it installs, and asks before adopting a bare tool name (owner ruling
  2026-09-28; two halves). **1. BUILT** (and it closes #105's residual, which is the same defect
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
  - **2. NOT BUILT, and the gap is bigger than it looks — it is the owner's call, not a schedule.**
    The premise was confirmed first: a bare `htop` on a machine without it is *silently planned as
    an install today*, with no prompt. So the missing piece is the `ask`. It cannot go in the
    resolver — that runs for `plan`, `check` and `status` too, and a read-only command may not
    prompt (`plan`'s own doc has the rule) — so it belongs on the sync/apply path, and the model
    carries **no tag saying a given install came from a bare declaration**. Deriving one from
    `BareLock` is unreliable, because the lock is only written when resolution is recording. So
    half 2 needs a decision on where the question lives and on what "records the line anyway" means
    for a declaration no backend owns — both of which change behaviour a user notices. Left open
    deliberately rather than half-built: a `[declare] install = "ask"` that quietly behaved like
    `"always"` would be the worst outcome available. — ISSUE #94
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

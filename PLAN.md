# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58, #69, #70, #71, #34, #37, #75, #76, #77, #78, #35.

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
- [ ] #99 a harness section that stops running is invisible: `crash/groupkill` lost 10 checks on
  ubuntu and the run still went green. (High — there is no floor on *checks run*, which is the
  exact mirror of `CAUGHT_FLOOR`, and three of the five properties only the matrix can verify are
  driven by that section. **The per-section tally ships; the FLOOR does not** — see `##[14c]`)
- [ ] #96 CI: the 8 distro integration images are red on the container harness's `dir:` check (a
  check that never observes what it asserts). (High — the matrix is the only thing that reaches a
  real manager)
- [ ] #97 CI: `Advisories and licences` and the MSVC build are red on `main`; the MSVC one means
  the `#71` Windows code has never been compiled by anything. (High)
- [ ] #98 CI: the `why.md` unattached-rationale ratchet is red on `main` — 53 entries against a
  ceiling of 52, and it reds `Build for x86_64-unknown-linux-gnu` on every push. (High — the one
  red job no open issue named)
- [ ] #100 the `void` leg's failure list is a sample of the network: six of eleven backend
  failures name a backend that passed in the next run, and `uv`/`yarn` fail in every run inside
  the same red. (Medium — a list nobody can compare to the last run is not evidence)

## Phase 3 — Correctness Mediums
- [ ] #91 planner template_needs_update compares raw source to rendered target (sibling of the #69 read-back fix). (Low; unreachable today, wrong if reached)
- [ ] #79 forget_all wipes all caches, #80 probe storm, #81 Mutex across --help, #82 batch deadline, #83 vars JSON fragile, #84 fan-out uncapped, #85 pool race, #86 zip sum wraps, #87 dir-symlink Windows.
- [ ] #25 lifecycle jobs on distro containers, #53 nimble Windows, #54 dirty-host fixtures, #55 gentoo jq.
- [ ] Suite-isolation bugs (Rust suite green only on NOPASSWD-sudo/adoptable hosts): #88 mock-layer tests probe real sudo, #89 guard-reachability control builds vacuous fixture, #90 fan-out floor vs skewed hosts.
- [ ] #41 prose tax distill, #36 doc-comment layer, #38 eopkg RETIRED honesty.

## Phase 4 — Feature gaps (Info, after core is safe)
- [ ] Adopt discovery: #59 dotfiles, #60 schedules, #61 repos/PPAs, #62 shell, #63 settings, #64 services, #65 firewall.
- [ ] Export: #66 from-scan, #67 from-declaration.
- [ ] #72 nixos activation gap (CI leg), #73 search-index noun, #74 crown-jewel invariant.
- [ ] Interop/process: #42, #43, #44. Process: #17–#21 bundles (split per sub-item when working).

## Routing rule for new issues
Any AI opening an issue here MUST insert it into the phase above it belongs in (foundations → safety → correctness → features). Primitives (#69-style) and safety bypasses go in Phase 1–2 even if filed later. See AI_ISSUE_ROUTING.md.

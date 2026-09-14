# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58.

## SKIP — duplicates of one event, do not re-work
- #32 DUP of #23+#24; #48 DUP of #32; #49 DUP of #33; #45 DUP of #26 (same file:line, same fix).

## Phase 1 — Foundations first (primitives others compose into)
- [ ] #69 templated content (writeText analogue with per-user substitution). (Medium — unblocks #73)
- [ ] #70 secrets compose into templates (${secret:} flow). (Medium)
- [ ] #71 per-user home layer (manifests, ownership, idempotent dirs). (Medium)
- [ ] #34 converged fast-path duplication → route through Phase::all. (Medium)
- [ ] #37 proven-by-default polarity flip. (High)

## Phase 2 — Safety Criticals/Highs
- [ ] #75 download truncates live PATH binary → temp+rename. (Critical)
- [ ] #76 symlink guard bypass via parent. (High)
- [ ] #77 watch blind to profiles/vars. (High)
- [ ] #78 single info() failure aborts whole plan. (High)
- [ ] #35 Reaped self-attestation token. (High)
- [ ] #39 harness half-totem (mutation survival). (High)
- [ ] #40 two oracles that cannot fail. (Medium)

## Phase 3 — Correctness Mediums
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

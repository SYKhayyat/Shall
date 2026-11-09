#!/usr/bin/env bash
# smoke.sh — "is this tree sane right now?" in a couple of minutes, not half an hour.
#
# `release-check.sh` is the shippability gate: a release build, the whole suite, the Docker matrix.
# That is the right thing to run before tagging and the wrong thing to run while editing — so
# people run nothing, and the first time they learn a workflow file has drifted from an installer
# is on the nightly. This is the tier between "cargo check" and "release".
#
# What it runs, and why each one is here rather than in the full gate:
#
#   1. THE TREE BUILDS. A debug build of the current binary. Everything below that needs the
#      binary uses this one, so a failure here stops the run rather than producing four confusing
#      follow-on failures.
#   2. THE DOCUMENTS AGREE WITH THE REGISTER. `decision-count.sh --check` — seconds, and it catches
#      the drift that a hand-edited count causes every time.
#   3. THE HARNESSES' OWN PREDICATES. `harness-logic-test.sh` — the shell harnesses have their own
#      library, and it is not compiled by `cargo test`.
#   4. THE RELEASE/INSTALL SURFACE AGREES WITH ITSELF. The review tests that compare ci.yml's
#      matrix, the release job, install.sh and install.ps1 in both directions. This is the drift
#      #21 was filed about: a target added to the workflow and not to an installer, or a version
#      bumped in one place.
#   5. THE SMOKE SUBSET. A named set of test modules that exercise the command surface end to end
#      — dry-run previews, refusals, JSON documents, the exit-code policy — without the whole
#      suite. Chosen for "would catch a broken binary", not for coverage.
#
# **Every check reports whether it ran.** A check that is skipped says so and says why, because a
# silent skip is indistinguishable from a pass in a log nobody reads closely — that is the
# acceptance criterion this script exists to meet, and it is the same defect the integration
# harnesses were cured of.
#
# Usage:
#   ./scripts/smoke.sh                 # everything that can run on this host
#   ./scripts/smoke.sh --no-build      # skip step 1 (the binary is already built)
#   ./scripts/smoke.sh --list          # print the checks and the smoke module set, run nothing
#
# Exit code is non-zero if any check ran and failed. A skipped check is not a failure and is not
# counted as a pass either — the summary says how many of each.

set -u
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "$0")/.." && pwd))"
cd "$REPO_ROOT" || { echo "cannot cd to repo root"; exit 2; }

GREEN=""; RED=""; YEL=""; RST=""
if [ -t 1 ]; then
    GREEN="$(printf '\033[32m')"; RED="$(printf '\033[31m')"
    YEL="$(printf '\033[33m')";  RST="$(printf '\033[0m')"
fi

step()  { echo; echo "===== $* ====="; }
pass()  { echo "${GREEN}[PASS]${RST} $1"; passed=$((passed + 1)); echo "PASS  $1" >> "$LEDGER"; }
fail()  { echo "${RED}[FAIL]${RST} $1"; failed=$((failed + 1)); echo "FAIL  $1" >> "$LEDGER"; }
skip()  { echo "${YEL}[SKIP]${RST} $1 — $2"; skipped=$((skipped + 1)); echo "SKIP  $1 — $2" >> "$LEDGER"; }

# The smoke module set. Named prefixes, matched against the suite's full test paths.
#
# These are the modules whose whole job is to drive a real binary end to end: a preview that
# writes nothing, a refusal that cites a line, a JSON document a script can read, an exit code
# that means what the policy says. They are not the coverage — they are the part that fails when
# the binary is broken rather than when a detail is.
SMOKE_MODULES=(
    "e2e::dry_run_every_verb_tests"
    "e2e::grade2_info_tests"
    "e2e::grade4_refusal_names_the_line_tests"
    "e2e::grade7_protected_skip_is_reported_tests"
    "e2e::unknown_backend_family_tests"
    "core::exit_policy"
    "verbs::plan"
    "review::the_panic_sites_are_counted_and_not_just_read_past"
    "review::the_smoke_tier_reports_what_it_did_not_run"
)

LEDGER="$(mktemp)"
passed=0; failed=0; skipped=0
trap 'rm -f "$LEDGER"' EXIT

DO_BUILD=1
for arg in "$@"; do
    case "$arg" in
        --no-build) DO_BUILD=0 ;;
        --list)
            echo "checks:"
            echo "  1. the tree builds (debug)"
            echo "  2. the documents agree with the register"
            echo "  3. the harnesses' own predicates"
            echo "  4. the release/install surface agrees with itself"
            echo "  5. the smoke subset:"
            for m in "${SMOKE_MODULES[@]}"; do echo "       $m"; done
            exit 0
            ;;
        *)
            echo "unknown argument: $arg (try --list)" >&2
            exit 2
            ;;
    esac
done

# ---------------------------------------------------------------- 1. the tree builds
if [ "$DO_BUILD" = "1" ]; then
    step "1. THE TREE BUILDS"
    if cargo build 2>&1 | tail -5 | grep -q "^error"; then
        fail "cargo build"
        # Everything below needs the binary, so stopping is honest rather than producing four
        # follow-on failures that all say "the same thing went wrong".
        echo
        echo "${RED}stopping: nothing below can run without a build${RST}"
        echo
        echo "summary: $passed passed, $failed failed, $skipped skipped"
        exit 1
    fi
    pass "cargo build"
else
    skip "cargo build" "--no-build"
fi

# ------------------------------------------- 2. the documents agree with the register
step "2. THE DOCUMENTS AGREE WITH THE REGISTER"
if [ ! -f scripts/decision-count.sh ]; then
    skip "decision-count.sh --check" "the script is not in this tree"
elif bash scripts/decision-count.sh --check >/dev/null 2>&1; then
    pass "decision-count.sh --check"
else
    fail "decision-count.sh --check"
    bash scripts/decision-count.sh --check 2>&1 | grep -E "BAD|FAILED" | head -10
fi

# ------------------------------------------------ 3. the harnesses' own predicates
step "3. THE HARNESSES' OWN PREDICATES"
if [ ! -f scripts/harness-logic-test.sh ]; then
    skip "harness-logic-test.sh" "the script is not in this tree"
elif bash scripts/harness-logic-test.sh >/dev/null 2>&1; then
    pass "harness-logic-test.sh"
else
    fail "harness-logic-test.sh"
    bash scripts/harness-logic-test.sh 2>&1 | tail -15
fi

# -------------------------------- 4. the release/install surface agrees with itself
step "4. THE RELEASE/INSTALL SURFACE AGREES WITH ITSELF"
#
# The module #21 was filed about. It compares `ci.yml`'s matrix, its release job, `install.sh` and
# `install.ps1` in both directions, so a target added in one place and forgotten in another is a
# red check rather than a broken install on somebody's machine.
release_module="review::the_release_publishes_what_the_installers_ask_for_tests"
if cargo test --test suite "$release_module" -- --test-threads=1 >/dev/null 2>&1; then
    pass "$release_module"
else
    fail "$release_module"
    cargo test --test suite "$release_module" -- --test-threads=1 2>&1 | grep -E "panicked|assertion|FAILED" | head -10
fi

# ----------------------------------------------------------------- 5. the smoke subset
step "5. THE SMOKE SUBSET"
#
# One `cargo test` invocation per module rather than one filter, so a module that matches nothing
# is reported as a skip rather than passing silently as "0 tests ran, 0 failed".
for module in "${SMOKE_MODULES[@]}"; do
    if ! cargo test --test suite "$module" -- --list >/dev/null 2>&1; then
        skip "$module" "the module does not exist in this tree"
        continue
    fi
    ran="$(cargo test --test suite "$module" -- --list 2>/dev/null | grep -c ': test$')"
    if [ "$ran" -eq 0 ]; then
        skip "$module" "it matches no tests (a renamed or removed module)"
        continue
    fi
    if cargo test --test suite "$module" >/dev/null 2>&1; then
        pass "$module ($ran tests)"
    else
        fail "$module ($ran tests)"
        cargo test --test suite "$module" 2>&1 | grep -E "panicked|assertion|FAILED" | head -8
    fi
done

# ------------------------------------------------------------------------- summary
echo
echo "================= SUMMARY ================="
sort "$LEDGER" | sed 's/^/  /'
echo "-------------------------------------------"
echo "  $passed passed, $failed failed, $skipped skipped"
echo
if [ "$failed" -gt 0 ]; then
    echo "${RED}SMOKE FAILED${RST}"
    exit 1
fi
if [ "$skipped" -gt 0 ]; then
    echo "${YEL}SMOKE PASSED, with $skipped check(s) skipped — read the skips above${RST}"
    exit 0
fi
echo "${GREEN}SMOKE PASSED${RST}"

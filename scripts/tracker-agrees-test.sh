#!/usr/bin/env bash
# The disagreement `scripts/tracker-agrees.py` exists to catch, fed to it directly.
#
# A check that only ever runs against a healthy tree proves nothing: the eight stale items and the
# thirteen untracked issues both looked exactly like this one does — green. Each case below is a
# shape that was live on 2026-11-22, so a regression in the comparison fails here rather than in
# somebody's week.
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
check="$here/tracker-agrees.py"
fails=0

expect() { # expect <name> <wanted-exit> <tasks-json> <issues-json> [wanted-substring]
    local name="$1" want="$2" tasks="$3" issues="$4" needle="${5:-}" out code
    out="$(TASKS="$tasks" ISSUES="$issues" python3 "$check" 2>&1)"
    code=$?
    if [ "$code" -ne "$want" ]; then
        echo "FAIL  $name — exit $code, wanted $want"; echo "$out" | sed 's/^/      /'
        fails=$((fails + 1)); return
    fi
    if [ -n "$needle" ] && ! printf '%s' "$out" | grep -qF "$needle"; then
        echo "FAIL  $name — said nothing about \"$needle\""; echo "$out" | sed 's/^/      /'
        fails=$((fails + 1)); return
    fi
    echo "ok    $name"
}

t() { printf '[{"id":%s,"state":"todo","title":"%s"}]' "$1" "$2"; }
i() { printf '[{"number":%s,"title":"%s"}]' "$1" "$2"; }

expect "one item, one open issue" 0 "$(t 1 'gh #7: the seventh thing')" "$(i 7 'the seventh thing')" "1 open item(s) for 1 open issue(s)"

# The slow drift: the issue shipped and the item stayed open.
expect "an item whose issue is closed" 1 "$(t 1 'gh #7: done upstream')" '[]' "which is closed or does not exist"

# The invisible direction: the issue is open and nobody is tracking it here.
expect "an open issue with no item" 1 '[]' "$(i 9 'untracked')" "tracked by no item"

# A title that mentions an issue without naming it was how six items drifted.
expect "a title that does not begin with gh #NNN" 1 "$(t 1 'see issue #7 for this')" "$(i 7 'x')" "names no issue"

# Done items are not claims about the tracker, so they are not compared at all.
expect "a done item whose issue is closed" 0 \
    '[{"id":1,"state":"done","title":"gh #7: shipped"}]' '[]' "0 open item(s)"

expect "no list handed over is a skip, not a pass" 2 "" "" "SKIP"

echo
if [ "$fails" -gt 0 ]; then
    echo "TRACKER-AGREES TEST FAILED ($fails)"
    exit 1
fi
echo "TRACKER-AGREES TEST PASSED"

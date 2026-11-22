#!/usr/bin/env bash
# Does the branch's task list agree with the tracker?
#
# The rule is one-to-one: an open `comtai` item is a claim that an issue is open, named in its
# title as `gh #NNN:`; an open issue is a claim that somebody is tracking it. Neither was true on
# 2026-11-22 — eight items sat open on issues that had shipped, thirteen open issues had no item
# at all — and nothing could have noticed, because the two lists are different tools and no check
# read both.
#
# Exit codes, because `scripts/smoke.sh` maps them rather than parsing prose:
#   0  the lists agree
#   1  they do not (every disagreement is printed)
#   2  skipped — this host cannot ask one side, and the reason is on stdout
#
# A skip is not a pass. `gh` needs state and `comtai` needs the branch's list; on a machine with
# neither, this check has verified nothing and says so.
set -uo pipefail

say_skip() { echo "SKIP: $1"; exit 2; }

command -v comtai >/dev/null 2>&1 || say_skip "comtai is not installed here"
command -v gh >/dev/null 2>&1 || say_skip "gh is not installed here"
command -v python3 >/dev/null 2>&1 || say_skip "python3 is not installed here"
gh auth status >/dev/null 2>&1 || say_skip "gh is not authenticated here, so the tracker cannot be read"

# The whole list, including done, and the shape of each field, comes from comtai itself: parsing
# `comtai ls`'s column layout would make a presentation change look like a tracking failure.
tasks_json="$(comtai ls --all --json 2>/dev/null)" || say_skip "comtai could not list this branch"
[ -n "$tasks_json" ] || say_skip "comtai returned no list"

issues="$(gh issue list --state open --limit 400 --json number,title 2>/dev/null)" ||
    say_skip "gh could not list this repository's issues"
[ -n "$issues" ] || say_skip "gh list the tracker once, or there is nothing open"

export TASKS="$tasks_json" ISSUES="$issues"
# shellcheck disable=SC2154
exec python3 "$(dirname "$0")/tracker-agrees.py"

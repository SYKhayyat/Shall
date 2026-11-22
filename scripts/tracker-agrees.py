#!/usr/bin/env python3
"""The comparison `scripts/tracker-agrees.sh` runs, kept separate so it can be tested without a tracker.

Inputs arrive as JSON in `TASKS` and `ISSUES`, so the test feeds the disagreement it is about
rather than describing one. Exit 1 with one `FAIL:` line per disagreement, 0 with a summary.
"""
import json, os, re, sys

if not os.environ.get("TASKS") or not os.environ.get("ISSUES"):
    print("SKIP: no list was handed over, so nothing was compared")
    sys.exit(2)

tasks = [t for t in json.loads(os.environ["TASKS"]) if t.get("state") != "done"]
issues = {i["number"]: i["title"] for i in json.loads(os.environ["ISSUES"])}

# `gh #NNN:` and nothing else. A title that merely mentions an issue somewhere is what let six
# items and thirteen issues drift apart while both lists looked complete.
named = re.compile(r"^gh #(\d+):")

bad = []
claimed = {}
for t in tasks:
    m = named.match(t["title"])
    if not m:
        bad.append(
            f"comtai #{t['id']} \"{t['title']}\" names no issue — a title begins `gh #NNN:`"
        )
        continue
    n = int(m.group(1))
    claimed.setdefault(n, []).append(t["id"])
    if n not in issues:
        bad.append(
            f"comtai #{t['id']} is about #{n}, which is closed or does not exist —"
            " close the item or reopen the issue"
        )

for n, ids in sorted(claimed.items()):
    if n in issues and len(ids) > 1:
        # Not a defect: #211 is two halves. Reported so the pairing stays legible.
        print(f"note: #{n} is tracked by {len(ids)} items ({', '.join('#' + str(i) for i in ids)})")

for n in sorted(issues):
    if n not in claimed:
        bad.append(
            f"#{n} \"{issues[n]}\" is open and tracked by no item — file one,"
            " or close the issue if it is done"
        )

if bad:
    for line in bad:
        print(f"FAIL: {line}")
    sys.exit(1)

print(f"ok: {len(tasks)} open item(s) for {len(issues)} open issue(s)")

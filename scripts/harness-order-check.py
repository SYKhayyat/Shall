#!/usr/bin/env python3
"""A harness may not call a function before the line that defines it.

Bash resolves a function name when the call *executes*, so a helper defined below the
section that calls it is `command not found` at runtime — with the harness's own error
handling turning that into a verdict about Shall. On the 2026-11-24 Windows nightly
(`45171335650`) it read:

    scripts/integration-windows.sh: line 1121: removal_leaves_binary: command not found
      FAIL  nimjson binary gone after uninstall (it is still there)

`removal_leaves_binary` was defined at line 1646 and called at 1121. Its whole purpose was
to score that nilble leftover as *named, therefore fine* (#136, C11), and because the call
came first the fix never ran once. The check is per-file and static: it does not run the
harness, so it costs seconds and it is in every tier that runs the harness predicates.

**The rule needs the call graph, not just line order.** A call inside a function body is
fine when the callee is defined lower down *provided the caller is not invoked until later*
— `canary` is defined at 1450 and calls `assert_binary_gone` at 1671's definition, which is
sound because `canary` is first invoked at 2,000-odd. So each function gets the earliest
line at which it can execute (its first top-level invocation, or the earliest invocation of
whatever calls it), and a call is early only when it fires before the callee's definition.

Usage: harness-order-check.py FILE [FILE ...]
"""

import re
import sys

# A command position: start of line, or after one of these, at the start of a simple command.
LEAD = re.compile(
    r"^\s*(?:if|then|else|elif|while|until|do|!|\{|\()?\s*"
)
SPLIT = re.compile(r"\|\||&&|[|;(]|\$\(|\{|\bthen\b|\bdo\b|\belse\b")
DEF = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\(\)\s*\{")
IDENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)$")


def strip_comment(line: str) -> str:
    """Remove a trailing shell comment, leaving `#` inside quotes alone."""
    out, quote = [], None
    for i, c in enumerate(line):
        if quote:
            out.append(c)
            if c == quote:
                quote = None
            continue
        if c in "\"'":
            quote = c
            out.append(c)
            continue
        if c == "#" and (i == 0 or line[i - 1] in " \t;|&("):
            break
        out.append(c)
    return "".join(out)


def code_depths(lines):
    """Brace depth at the end of each line, with strings and comments removed.

    The state — inside a `"…"` or `'…'` — carries across lines, because `hard()` in both
    harnesses opens a string on one line and closes it on the next, with a `}` inside it. A
    per-line scan read that `}` as code, lost the balance, and then reported the function as
    never closed.
    """
    depth = 0
    quote = None
    out = []
    for line in lines:
        i = 0
        while i < len(line):
            c = line[i]
            if quote:
                if c == "\\":
                    i += 2
                    continue
                if c == quote:
                    quote = None
                i += 1
                continue
            if c in "\"'":
                quote = c
            elif c == "#" and (i == 0 or line[i - 1] in " \t;|&("):
                break
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
            i += 1
        out.append(depth)
    return out


def analyse(path: str):
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    depth_at = code_depths(lines)

    # --- definitions: `name() {`, body to the line that closes its brace ---
    #
    # Counted from the running depth, not found by "the next `}` in column 0": a function whose
    # body closed with an indented brace swallowed every definition below it, and the file then
    # reported a call to a function it had refused to record as defined — the two halves
    # disagreeing in silence.
    defs = {}  # name -> definition line (1-based)
    bodies = {}  # name -> (first line, last line)
    for idx, line in enumerate(lines):
        m = DEF.match(line)
        if not m:
            continue
        name = m.group(1)
        before = depth_at[idx - 1] if idx else 0
        defs.setdefault(name, idx + 1)
        if depth_at[idx] <= before:  # a one-line body, closed on its own line
            bodies.setdefault(name, (idx + 1, idx + 1))
            continue
        end = idx + 1
        while end < len(lines) and depth_at[end] > before:
            end += 1
        bodies.setdefault(name, (idx + 1, end if end < len(lines) else len(lines)))

    def enclosing(lineno: int):
        for name, (lo, hi) in bodies.items():
            if lo <= lineno <= hi:
                return name
        return None

    # --- call sites: a known function name used as a command ---
    def sites(text: str):
        """Every known function name standing at a command position in `text`."""
        found = []
        for part in SPLIT.split(text):
            rest = part.strip()
            m = LEAD.match(part)
            if m:
                rest = part[m.end():].strip()
            if not rest:
                continue
            word = IDENT.match(rest.split()[0])
            if word:
                found.append(word.group(1))
        return found

    calls = []  # (callee, line, caller or None)
    for n, raw in enumerate(lines, start=1):
        text = strip_comment(raw)
        if not text.strip():
            continue
        m = DEF.match(text)
        if m:
            name = m.group(1)
            # A one-liner closes its body on the same line, so the calls in that body are on
            # this line and belong to this function. Skipping whole definition lines is how the
            # first version of this check passed a file whose only call sat inside one.
            tail = text.split("{", 1)[1]
            for callee in sites(tail):
                if callee in defs and callee != name:
                    calls.append((callee, n, name))
            continue
        for callee in sites(text):
            if callee in defs:
                caller = enclosing(n)
                if callee == caller:
                    continue  # recursion is not a definition-order question
                calls.append((callee, n, caller))

    # --- earliest line at which each function can execute (relaxation) ---
    INF = float("inf")
    earliest = {name: INF for name in defs}
    for callee, line, caller in calls:
        if caller is None:
            earliest[callee] = min(earliest[callee], line)
    for _ in range(len(defs) + 1):
        changed = False
        for callee, _, caller in calls:
            if caller is None:
                continue
            seen = earliest[caller]
            if seen < earliest[callee]:
                earliest[callee] = seen
                changed = True
        if not changed:
            break

    problems = []
    for callee, line, caller in calls:
        fires = line if caller is None else earliest[caller]
        if fires == INF:
            continue  # this caller never runs from this file; the definition order is moot
        if fires < defs[callee]:
            where = "at top level" if caller is None else f"from `{caller}`"
            problems.append(
                f"  FAIL  {path}:{line}: `{callee}` is called {where}, and it is defined at "
                f"line {defs[callee]} — bash resolves the name when the call runs, so this is "
                f"`{callee}: command not found` and the harness turns it into a verdict"
            )
    return problems


def main(argv):
    if not argv:
        print("usage: harness-order-check.py FILE [FILE ...]")
        return 2
    bad = 0
    for path in argv:
        found = analyse(path)
        if found is None:
            bad += 1
            continue
        for line in found:
            print(line)
            bad += 1
    if not bad:
        print("  ok    every harness function is defined before the line that can first call it")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

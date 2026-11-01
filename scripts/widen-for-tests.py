#!/usr/bin/env python3
"""Widen, for the tests' sake, whatever the compiler says a moved test needs.

**The compiler names the item, and this script does the rest.** A relocated test
sees the crate's public API, so an item it reaches must become visible — and the
item is known exactly where the compiler reports it. Doing that by hand is
ninety percent of the sweep's per-file cost, and it is the part that has no
judgement in it.

What it must never do: **widen something a ruling protects.** A `#[cfg(test)]`
constructor that must not ship (C3) is not an item this script touches — it is
not named by the compiler, because a test crate cannot see it to *want* it, and
the exception table is where such a file is recorded instead.

Usage: python3 scripts/widen-for-tests.py src/core/executor.rs
"""
import re
import subprocess
import sys
from pathlib import Path

NOTE = (
    "/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a\n"
    "/// test crate links the library without `cfg(test)`, so a private item is invisible\n"
    "/// there.\n"
)
FIELD_NOTE = (
    "/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and\n"
    "/// a test crate links the library without `cfg(test)`, so a private field is invisible\n"
    "/// there.\n"
)


def build_errors():
    """The unresolved/private names the compiler reports, as (kind, name)."""
    out = subprocess.run(
        ["cargo", "build", "--all-targets"], capture_output=True, text=True
    )
    text = out.stderr
    found = []
    # `cannot find type/value/function `X` in this scope` and its `no X found` cousins
    for m in re.finditer(
        r"cannot find (?:type|value|function|module or crate|macro|struct, variant or union type)"
        r" `(\w+)`",
        text,
    ):
        found.append(("item", m.group(1)))
    # `function `X` is private` / `associated function `X` is private`
    for m in re.finditer(r"(?:function|associated function|method|associated const|static) `(\w+)` is private", text):
        found.append(("fn", m.group(1)))
    # `field `X` of struct `Y` is private`
    for m in re.finditer(r"field `(\w+)` of struct `([\w:]+)` is private", text):
        found.append(("field", m.group(1)))
    return found, out.returncode


def struct_body_ranges(text: str) -> list[tuple[int, int]]:
    """The line ranges that are struct *declarations*, as `(start, end)`.

    **A field appears in two places**: once in a struct's body, and once in an
    initialiser for it. The first is the one to widen; the second is an expression,
    and `pub ` there is a syntax error. So the ranges are found first, by matching
    each `struct X {` to its closing brace.
    """
    ranges = []
    for m in re.finditer(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+\s*\{", text):
        start = text.count("\n", 0, m.start())
        depth, i = 1, m.end()
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        ranges.append((start, text.count("\n", 0, i) + 1))
    return ranges


def widen(path: Path, name: str, kind: str) -> bool:
    """Widen the item the compiler named, and say why where it is declared."""
    joined = path.read_text()
    lines = joined.splitlines(keepends=True)
    struct_ranges = struct_body_ranges(joined)
    for i, line in enumerate(lines):
        stripped = line.strip()
        # Skip comments. **`pub(crate)` is not public to a test crate** — a test crate
        # links the library as an outsider, so those are widened too rather than skipped.
        if stripped.startswith("//"):
            continue
        if stripped.startswith("pub ") and not stripped.startswith("pub("):
            # **A public struct can still have a private field**, so the skip does not
            # apply when the compiler has named a field rather than an item.
            if not (
                kind == "field"
                and re.match(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+\s*\(", line)
            ):
                continue

        # A visibility qualifier may precede the name, and is not part of the match.
        m = re.match(
            r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?"
            r"(?P<kind>fn|struct|enum|const|static|type|trait)\s+(?P<name>"
            + re.escape(name)
            + r")\b",
            line,
        )
        if m and kind in ("fn", "item"):
            indent = m.group("indent")
            rest = re.sub(r"^pub(?:\([^)]*\))?\s+", "", stripped)
            lines[i] = f"{indent}pub {rest}\n"
            _note(lines, i, indent, NOTE)
            path.write_text("".join(lines))
            return True

        # **A positional field is widened on the tuple struct that holds it.** The
        # compiler names it `field 0 of struct X`, and the declaration is
        # `pub struct X(Option<…>);` — no field name to match, and the *struct* being
        # public is no reason to think its field is.
        m = re.match(
            r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+\s*\(", line
        )
        if m and kind == "field" and name == "0":
            indent = m.group("indent")
            rest = re.sub(r"^(pub(?:\([^)]*\))?\s+)?", "", stripped)
            lines[i] = f"{indent}pub {rest}\n"
            j = i - 1
            while j >= 0 and lines[j].strip().startswith(("///", "//")):
                j -= 1
            note = "".join(indent + l for l in FIELD_NOTE.splitlines(keepends=True))
            lines[j + 1:j + 1] = [note]
            path.write_text("".join(lines))
            return True

        # **A named field is widened only inside a struct's body**, never inside an
        # initialiser for the struct it belongs to — that one is an expression, and
        # `pub` there is a syntax error.
        m = re.match(
            r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s*)?" + re.escape(name) + r"\s*:",
            line,
        )
        if m and kind == "field":
            if not any(lo <= i <= hi for lo, hi in struct_ranges):
                continue
            indent = m.group("indent")
            rest = re.sub(r"^pub(?:\([^)]*\))?\s+", "", stripped)
            lines[i] = f"{indent}pub {rest}\n"
            _note(lines, i, indent, FIELD_NOTE)
            path.write_text("".join(lines))
            return True
    return False


def _note(lines, i: int, indent: str, note: str) -> None:
    """Put the reason into the item's doc comment, or above the item."""
    j = i - 1
    doc = []
    while j >= 0 and lines[j].strip().startswith("///"):
        doc.append(lines[j])
        j -= 1
    body = "".join(
        indent + l if l.strip() else l for l in note.splitlines(keepends=True)
    )
    if doc:
        lines[j + 1:j + 1] = [body]
    else:
        lines[i:i] = [body]


def find_defining_file(name: str, kind: str, start: Path) -> Path | None:
    """The file that declares an item the compiler named.

    **An item does not have to live in the file being swept.** A test reaches a
    helper its src file *imported* — `has_bytes` comes from `core::launch` — so when
    the name is not in the file being swept, the declaration is found by grep and
    widened where it lives.
    """
    out = subprocess.run(
        ["grep", "-rl", "--include=*.rs", "-E",
         r"^\s*(pub(\([^)]*\))?\s+)?(async\s+)?(fn|struct|enum|const|static|type|trait)\s+"
         + re.escape(name) + r"\b",
         "src/"],
        capture_output=True, text=True,
    )
    files = [Path(f) for f in out.stdout.split()]
    for f in files:
        if f != start:
            return f
    return None


def main():
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "src/core/executor.rs")
    for _round in range(12):
        found, rc = build_errors()
        if rc == 0:
            print("the build is clean")
            return 0
        progressed = False
        for kind, name in dict.fromkeys(found):
            if name in ("", "super", "crate", "self"):
                continue
            if widen(path, name, kind):
                print(f"widened {kind} `{name}` in {path}")
                progressed = True
                continue
            # not in the file being swept: find where it lives and widen it there
            other = find_defining_file(name, kind, path)
            if other and widen(other, name, kind):
                print(f"widened {kind} `{name}` in {other}")
                progressed = True
        if not progressed:
            print("nothing left to widen; the remaining errors need a human")
            return 1
    print("still not clean after twelve rounds")
    return 1


if __name__ == "__main__":
    sys.exit(main())

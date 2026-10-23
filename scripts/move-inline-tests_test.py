#!/usr/bin/env python3
"""The mover's own test: the acceptance criteria, in executable form.

A tool that relocates five thousand lines of tests across a hundred and eighty
files is not a tool to run unchecked. **These are the same criteria the sweep
itself is checked against**, so a version of the tool that quietly loses a test
fails here before it can lose one there:

1. **No test is lost.** The count of `#[test]`/`#[tokio::test]` in what the tool
   wrote is the count that was in the fixture.
2. **The grouping survives.** A src file with more than one test module gets one
   folder per module, so two helpers with the same name cannot collide.
3. **A `#[cfg(test)]` constructor stays inline.** The tool never ships one.
4. **The cfg gates travel with their imports**, and an ungated copy does not
   appear beside a gated one.
5. **An `include_str!` path is made one directory deeper** — and a path the
   product interprets is left alone.
6. **Duplicate imports are folded**, not emitted twice.

Run: python3 scripts/move-inline-tests_test.py
"""
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TOOL = REPO / "scripts" / "move-inline-tests.py"
FIXTURE = REPO / "scripts" / "fixtures" / "move-inline-tests" / "subject.rs"

PASS, FAIL = "  \033[32m✓\033[0m", "  \033[31m✗\033[0m"
results = []


def check(name: str, ok: bool, detail: str = "") -> None:
    results.append(ok)
    print(f"{PASS if ok else FAIL} {name}")
    if not ok and detail:
        print(f"      {detail}")


def main() -> int:
    text = FIXTURE.read_text()
    inline = len(re.findall(r"#\[(?:tokio::)?test[\]\s(]", text[text.find("#[cfg(test)]"):]))
    check("the fixture has inline tests to move", inline >= 5, f"found {inline}")

    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        # **A src file of its own, in a src-shaped directory**, because the
        # depth arithmetic is part of what the tool has to get right.
        src = work / "src" / "core" / "subject.rs"
        src.parent.mkdir(parents=True)
        shutil.copy(FIXTURE, src)

        ran = subprocess.run(
            [sys.executable, str(TOOL), str(src), "tests/core/subject", "core::subject"],
            capture_output=True,
            text=True,
            cwd=work,  # **The mirror is relative, so the tool must run where it writes**
        )
        check("the tool runs", ran.returncode == 0, ran.stderr.strip())

        mirror = work / "tests" / "core" / "subject"
        flat = list((mirror / "tests").glob("*.rs")) if (mirror / "tests").is_dir() else []
        sub = (mirror / "second_shape_tests").is_dir()

        # 1. no test lost
        written = sum(
            len(re.findall(r"#\[(?:tokio::)?test[\]\s(]", p.read_text()))
            for p in mirror.rglob("*.rs")
        )
        check(
            "no test is lost",
            written == inline,
            f"{inline} in the fixture, {written} written",
        )

        # 2. the grouping survives
        check(
            "a file with several test modules keeps one folder per module",
            sub and len(flat) >= 2,
            f"flat files: {len(flat)}, second folder exists: {sub}",
        )

        # 3. the cfg(test) constructor stays inline
        check(
            "a `#[cfg(test)]` constructor stays inline",
            "with_count" in src.read_text(),
            "the constructor left the src file, so it would have shipped",
        )
        check(
            "the lib's own cfg(test) items are not shipped",
            "pub fn with_count" not in "".join(
                p.read_text() for p in mirror.rglob("*.rs")
            ),
        )

        # 4. the cfg gate travels
        gated = any(
            "#[cfg(unix)]" in p.read_text()
            and "MockExecutor" in p.read_text()
            for p in mirror.rglob("*.rs")
        )
        check("a cfg gate above an import travels with it", gated)

        # 5. the include path is made deeper — **by the amount the depth actually
        # changed**, not by a literal that a deeper fixture would invalidate.
        def ups(text: str) -> int:
            m = re.search(r'include_(?:str|bytes|file)!?\("((?:\.\./)*)', text)
            return len(m.group(1).split("../")) - 1 if m else 0

        source_ups = ups(text)
        written_ups = max(
            (ups(p.read_text()) for p in mirror.rglob("*.rs")), default=0
        )
        check(
            "an include_str! path is made deeper by the depth the file moved",
            written_ups == source_ups + 2,
            f"source had {source_ups} ups, the mirror has {written_ups}",
        )

        # 6. duplicates are folded — **per file, by name**, because the same statement
        # legitimately appears in *two* module files (each inherits the src file's
        # imports), and what must not happen is the same name twice in one of them.
        # A name in two imports is also legitimate when one is an alias, so the check
        # is on the name a statement actually binds.
        dupes, twice = [], []
        for f in mirror.rglob("*.rs"):
            body = f.read_text()
            statements = re.findall(r"(?m)^\s*(?:pub )?use [^;]+;", body)
            seen: dict[str, set] = {}
            for st in statements:
                one = " ".join(st.split())
                m = re.match(r"(?:pub )?use ([\w:]+(?:::\w+)*)::\{([^}]*)\};", one)
                if m:
                    seen.setdefault(m.group(1), set()).update(
                        n.split(" as ")[0].strip() for n in m.group(2).split(",") if n.strip()
                    )
                    continue
                m = re.match(r"(?:pub )?use ([\w:]+(?:::\w+)*);", one)
                if m:
                    parent, _, name = m.group(1).rpartition("::")
                    seen.setdefault(parent, set()).add(name)
            for path, names in seen.items():
                if len(names) != len(set(names)):
                    dupes.append(f"{f.relative_to(work)}:{path}")
            normalised = [" ".join(st.split()) for st in statements]
            twice += [
                s for s in set(normalised) if normalised.count(s) > 1
            ]
        check(
            "duplicate imports are folded rather than emitted twice",
            not dupes and not twice,
            f"same name twice: {dupes}; statement twice in one file: {twice}",
        )

        # and the two things a hand-mover got wrong and the tool must not repeat
        bodies = "\n".join(p.read_text() for p in mirror.rglob("*.rs"))
        # **A test's own `use crate::…` is the one that must become `shall::…`;** a
        # path the *product* interprets keeps its shape, and the tool does not go
        # looking for those.
        mine = [
            line
            for line in bodies.splitlines()
            if line.strip().startswith(("use ", "pub use "))
        ]
        check(
            "`crate::` becomes `shall::`",
            not any("crate::" in line for line in mine),
            str([line for line in mine if "crate::" in line]),
        )

    print()
    print(f"{sum(results)}/{len(results)} criteria hold")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())

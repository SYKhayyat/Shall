//! Can a branch of the archive extractor total declared sizes with arithmetic that wraps?
//!
//! `[config] max_unpacked_bytes` is the one thing between a download and an unbounded write, and
//! it is checked against numbers **the archive itself supplies**: for a zip, the sum of each
//! member's declared uncompressed size, read straight out of the central directory. Every one of
//! those fields was written by whoever built the file, so the addition is inside the trust
//! boundary — and `Iterator::sum` for integers **wraps in a release build** (and panics in a
//! debug one). Two members declaring `u64::MAX` and `2` total to `1` there, which is under any
//! cap anybody can configure, so the bound said yes to an archive declaring more bytes than the
//! address space.
//!
//! **The measurement, and why this file is a scan and not a test of the arithmetic.** The tar
//! branch walked its entries with `saturating_add`; the zip branch called `.sum()`. The rule is
//! now a named pair — `add_unpacked` and `declared_unpacked_total` in `src/utils/archive.rs` —
//! and `the_unpacked_total_saturates_rather_than_wrapping` in that file's own `#[cfg(test)]` module
//! drives it with the numbers a crafted archive carries. That module is in `src/` because these
//! two functions are private to the crate, and `a_writer_that_reaches_the_disk_goes_through_one`
//! is the precedent for putting the rest here: **an integration test cannot call a crate-private
//! rule, so it reads the source instead.**
//!
//! **Why the scan is worth having at all.** The defect was never in the arithmetic being hard to
//! write; it was in there being written twice, four lines apart, in two branches. A named rule
//! plus this scan means the second spelling is a build failure rather than a release-only
//! difference nobody can reproduce — and the scan is driven against the deleted line as its own
//! control, because a prohibition that cannot fail is a gate reporting green having examined
//! nothing.

use std::path::PathBuf;

fn archive_rs() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/utils/archive.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("reading {}: {e}", p.display()))
}

#[test]
fn no_branch_of_the_extractor_totals_a_size_with_a_bare_sum() {
    let src = archive_rs();

    // The scan must see the file, or "no match" is the passing answer for a scan that examined
    // nothing. Both branches are named, so this fails if either stops going through the rule.
    assert!(
        src.contains("add_unpacked(expanded, entry.size())"),
        "the tar branch no longer totals through add_unpacked; the scan below is looking for a \
         rule this file has stopped using"
    );
    assert!(
        src.contains("declared_unpacked_total("),
        "the zip branch no longer totals through declared_unpacked_total; ditto"
    );
    assert!(
        src.contains("fn add_unpacked(") && src.contains("fn declared_unpacked_total("),
        "the named rule is gone from src/utils/archive.rs and nothing in this file can say so"
    );

    for (n, line) in src.lines().enumerate() {
        let line = line.trim();
        // The rule's own definition, and the prose that explains it, are the two legitimate
        // mentions of the shape.
        if line.starts_with("//") || line.starts_with("///") || line.starts_with("fn add_unpacked")
        {
            continue;
        }
        assert!(
            !line.contains(".sum::<u64>()") && !line.contains(".sum()"),
            "src/utils/archive.rs:{} totals a size with a bare `.sum()`, which wraps in a release \
             build: {line}\n\nEvery declared size in an archive is a field its author wrote, so \
             this is arithmetic inside the trust boundary. Use `add_unpacked` (one entry at a \
             time) or `declared_unpacked_total` (a collection).",
            n + 1
        );
    }

    // **The control.** The line this issue is about, fed to the same predicate the loop uses, so
    // the scan is known to be capable of failing before it is believed.
    let before = "        let declared: u64 = (0..archive.len()).map(|i| f.size()).sum();";
    assert!(
        before.contains(".sum()"),
        "the control no longer describes the code this test was written for, so the loop above is \
         asserting against a fiction"
    );
}

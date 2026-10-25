use super::dated::{dating_of, Dating};
use crate::config::grammar::{GrammarError, Options, Origin};
use chrono::{DateTime, Utc};

/// One resolved declaration and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    pub options: Options,
    pub origin: Origin,
    /// `false` for an `absent:` line. The only genuinely new thing the desired-state map
    /// could not already carry (SPEC Phase 2).
    pub present: bool,
}

/// Decide between two declarations of the same package (SPEC II.7 rules 5 and 6).
///
/// **Two active declarations that contradict = ERROR.** Not first-wins, not file order.
/// Files used to be read in filesystem order and the first declaration won, so
/// `a.txt: jq@1.6` versus `b.txt: jq@1.7` was decided by the disk — and sorting the read
/// order only makes the wrong answer deterministic (V.5).
///
/// The one exception is a dated line: while it is counting it beats an undated one, and
/// once its date passes it stops counting entirely (II.7 rule 6). That is what makes
/// "take the game away until the weekend" expressible without it reading as a
/// contradiction with a timer (V.37).
pub fn reconcile(
    key: &str,
    a: Declared,
    b: Declared,
    now: DateTime<Utc>,
) -> Result<Declared, GrammarError> {
    let (da, db) = (dating_of(&a.options, now), dating_of(&b.options, now));

    // A lapsed line has no opinion. It lingers in your file — Shall must not rewrite what
    // you wrote (II.16) — but it does not participate.
    match (da, db) {
        (Dating::Lapsed, Dating::Lapsed) => return Ok(a),
        (Dating::Lapsed, _) => return Ok(b),
        (_, Dating::Lapsed) => return Ok(a),
        _ => {}
    }

    // Rule 6's exception to rule 5: while it is counting, a dated line beats an undated
    // one. This is the ONLY case where two contradicting declarations do not error.
    match (da.is_dated(), db.is_dated()) {
        (true, false) => return Ok(a),
        (false, true) => return Ok(b),
        _ => {}
    }

    if !contradicts(&a, &b) {
        // Identical, or one merely adds options the other did not set. Merge rather than
        // complain: declaring the same thing twice is not a disagreement.
        return Ok(merge(a, b));
    }

    Err(conflict_error(key, &a, &b))
}

/// Whether two declarations actually disagree, as opposed to merely both existing.
fn contradicts(a: &Declared, b: &Declared) -> bool {
    if a.present != b.present {
        return true;
    }
    // Same key set with different values is a disagreement; a key only one of them sets is
    // not.
    for (k, av) in a.options.iter() {
        if let Some(bv) = b.options.all(k).first() {
            if av.first() != Some(bv) {
                return true;
            }
        }
    }
    false
}

fn merge(mut a: Declared, b: Declared) -> Declared {
    for (k, vs) in b.options.iter() {
        if !a.options.contains(k) {
            // `insert`, not `set`: this loop copies every value of a key, and `set` would keep
            // only the last one — a merged declaration losing all but one of its `requires`.
            for v in vs {
                a.options.insert(k, v.clone());
            }
        }
    }
    a
}

/// The error names BOTH files. An error that names one of them is an error that blames the
/// wrong file half the time.
fn conflict_error(key: &str, a: &Declared, b: &Declared) -> GrammarError {
    let describe = |d: &Declared| -> String {
        if !d.present {
            return format!("absent:{}", key);
        }
        let opts: Vec<String> = d
            .options
            .iter()
            .flat_map(|(k, vs)| vs.iter().map(move |v| format!("{}={}", k, v)))
            .collect();
        if opts.is_empty() {
            key.to_string()
        } else {
            format!("{}@{}", key, opts.join(","))
        }
    };

    GrammarError::new(
        a.origin.clone(),
        format!(
            "`{}` is declared two different ways, and both are active:\n  \
             {}  says  {}\n  {}  says  {}",
            key,
            a.origin,
            describe(a),
            b.origin,
            describe(b),
        ),
    )
    .with_hint(
        "Shall will not pick one for you: whichever it chose would be right half the time \
         and silent about it. Change one of them, or put them in profiles that are not \
         active together.",
    )
}

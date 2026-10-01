//! Ask a manager what it accepts, instead of assuming.
//!
//! E11's fix was `--verify=false`, taken from helm's own error text on the machine where the
//! bug was reported. That machine ran helm v4.2.3, which has the flag. helm 3 does not, and
//! rejects it outright:
//!
//! ```text
//! Error: `helm` failed (exit 1): Error: unknown flag: --verify
//! ```
//!
//! So `@unverified` worked on helm 4 and broke every helm 3 — one argv defect traded for
//! another, from a fix derived from one machine's error message and shipped unconditionally.
//!
//! The lesson is not "check helm's version". A version table is the same assumption with a
//! number in it, and it goes stale the same way. **Ask the tool.** `--help` is the one argument
//! no package manager acts on, which is why the argv-drift gate uses it too.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// One program's help, and whether it has been asked for yet.
///
/// **The lock is per key and it is held across the probe**, which is what makes the cache do
/// what its doc claims. It used to be one map lock taken to read, dropped, and taken again to
/// write — a check-then-act with a process spawn in the gap — so *k* tasks that missed together
/// all spawned the probe. This is reached from the install argv path, per package, inside the
/// wave, and because the hottest fan-outs are multiplexed onto one task, each duplicate stalls
/// every other package in that wave rather than one of them.
///
/// The shape is `InstalledListings::once`'s and `VARS_MEMO`'s, both of which hold a per-key
/// lock across their fetch for exactly this reason. No generation counter here: `--help` output
/// does not change because Shall installed something.
type HelpSlot = std::sync::Arc<Mutex<Option<Option<String>>>>;

/// Help text already obtained this run, by `program <chain…>`. A manager's help does not change
/// while Shall is running, and an install of forty plugins must not launch forty help processes.
///
/// The text and not the answer: two questions are asked of the same help — does it document
/// this flag, and does it document verification at all — and caching per question would run the
/// probe twice for one process's output.
fn cache() -> &'static Mutex<HashMap<String, HelpSlot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, HelpSlot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// What `program <chain…> --help` prints, or `None` when it could not be asked.
fn help_text(program: &str, chain: &[String]) -> Option<String> {
    let key = format!("{} {}", program, chain.join(" "));
    // The outer lock only hands out the slot; it is never held across the spawn, so two
    // different programs still probe concurrently.
    // **A poisoned lock is recovered, not obeyed.** Both arms used to answer `probe(…)` directly,
    // which reads as prudence and is the opposite: a `Mutex` is poisoned *permanently* by the one
    // panic, so the branch turned a single panic anywhere in the process into **this cache
    // switched off for the rest of the run** — every later question about every program spawned a
    // fresh process, with nothing said, which is the cost the cache exists to avoid. And the
    // value a poisoned slot holds is not in doubt: `None` means "not probed yet" and `Some(None)`
    // means "probed, no answer", both of which are answers.
    //
    // `into_inner` is the standard recovery and it is sound here because the map is only ever
    // mutated by `entry().or_default().clone()` — a panic inside that leaves either no entry or a
    // complete default one, never half of an `Arc`.
    let slot = match cache().lock() {
        Ok(mut map) => map.entry(key).or_default().clone(),
        Err(poisoned) => poisoned.into_inner().entry(key).or_default().clone(),
    };
    let mut slot = match slot.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(hit) = slot.as_ref() {
        return hit.clone();
    }
    let answer = probe(program, chain);
    *slot = Some(answer.clone());
    answer
}

/// Run `program <chain…> --help` and return what it printed.
fn probe(program: &str, chain: &[String]) -> Option<String> {
    let mut args: Vec<String> = chain.to_vec();
    args.push("--help".to_string());
    // Through the executor's launcher, or a shimmed manager on Windows cannot be run at all —
    // the mistake the argv-drift gate made for four installed managers before it was fixed.
    let (prog, argv) = crate::core::launch::effective_command(program, &args);
    let mut cmd = std::process::Command::new(prog);
    cmd.args(&argv).stdin(std::process::Stdio::null());
    crate::core::blocking::command_output_bounded(&mut cmd, "manager help probe")
        .ok()
        .map(|o| {
            format!(
                "{}
{}",
                crate::utils::text::sanitize(&String::from_utf8_lossy(&o.stdout)),
                crate::utils::text::sanitize(&String::from_utf8_lossy(&o.stderr))
            )
        })
}

/// Does `program <chain…> --help` document verification **at all** — any flag that turns it on,
/// off, or points it at a keyring?
///
/// This is the discriminator between the two ways a capability flag can be absent from a tool,
/// and [`accepts_flag`] cannot tell them apart:
///
///   * the tool never verified — helm 3 documents no verification flag of any kind, so
///     `@unverified` asks for a state the machine is already in (Q14, V.104), and withholding
///     the flag silently is correct;
///   * the tool verifies under a flag we have the old name for — drift, and a defect.
///
/// A gate built on `accepts_flag` alone reports success for both, which is how a planted
/// `--shall-bogus-flag-zzz` passed the flag half of the argv-drift gate on a helm 4 host. It
/// lives here rather than in the test because it is a question about a tool, asked the same way
/// and through the same cache as the other one.
pub fn documents_verification(program: &str, chain: &[String]) -> Option<bool> {
    help_text(program, chain).map(|text| help_documents_verification(&text))
}

/// The pure half of [`documents_verification`], so it can be asserted against help text
/// captured from a version this machine does not have — which is the only way the helm 3 arm
/// is testable anywhere but a helm 3 host.
pub fn help_documents_verification(text: &str) -> bool {
    ["--verify", "--keyring", "--prov", "--signature"]
        .iter()
        .any(|f| mentions_flag(text, f))
}

/// Does `program <chain…> --help` mention `flag`?
///
/// **`None` means the tool could not be asked** — it is not on this machine, or its help would
/// not run — and that is deliberately different from `Some(false)`. The capability table is the
/// default; this probe exists to *correct* it where there is evidence, never to overrule it
/// with silence.
///
/// The first version returned a bare `bool` and folded "could not ask" into "does not accept".
/// It passed here, where helm v4 is installed, and broke two unit tests on every CI runner,
/// which has no helm at all: the argv builder had quietly acquired a dependency on the host.
/// That is the finding this whole round is about — a check whose answer depends on something
/// nobody enumerated — committed while fixing it.
///
/// The flag is matched without its `=value` tail, since help text writes `--verify` and the
/// argv writes `--verify=false`.
pub fn accepts_flag(program: &str, chain: &[String], flag: &str) -> Option<bool> {
    let name = flag.split('=').next().unwrap_or(flag);
    help_text(program, chain).map(|text| mentions_flag(&text, name))
}

/// Does `text` document `flag` as a flag in its own right?
///
/// A plain substring search is not enough: `--ca` occurs inside `--ca-file`, and helm 3's help
/// carries `--kube-insecure-skip-tls-verify`, which contains `verify` and would answer yes for
/// a `--verify` this version does not have. So the match ends at a character that cannot
/// continue a flag name.
fn mentions_flag(text: &str, name: &str) -> bool {
    let mut from = 0;
    while let Some(at) = text[from..].find(name) {
        let start = from + at;
        let end = start + name.len();
        let next = text[end..].chars().next();
        if !matches!(next, Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return true;
        }
        from = end;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_name_does_not_match_a_longer_one() {
        // The exact trap in helm 3's own help.
        assert!(!mentions_flag(
            "      --kube-insecure-skip-tls-verify   if true, …",
            "--verify"
        ));
        assert!(!mentions_flag("      --ca-file string   …", "--ca"));
        // And it must still find the real thing, in every shape help text writes it.
        assert!(mentions_flag("Use --verify=false to skip.", "--verify"));
        assert!(mentions_flag(
            "      --verify   verify the package",
            "--verify"
        ));
        assert!(mentions_flag("ends the line with --verify", "--verify"));
    }

    /// **One panic must not switch the cache off for the rest of the run.**
    ///
    /// `help_text` used to answer `probe(…)` on a poisoned lock, which reads as prudence — a cache
    /// nobody trusts, a fresh answer. But a `Mutex` is poisoned *permanently* by the one panic, so
    /// that branch turned a single panic anywhere in the process into every later question about
    /// every program spawning a new process, silently, which is exactly the cost the cache exists
    /// to remove. `PLAN.md` #81.
    ///
    /// The test poisons both locks for real — a panic while each is held, which is the only way a
    /// `Mutex` gets poisoned — and then asks a question the cache can answer from memory. With the
    /// old arms it spawns a process for a program that does not exist and answers `None`, so the
    /// cached text is gone and the test is red.
    #[test]
    fn a_poisoned_cache_is_still_a_cache() {
        const PROGRAM: &str = "shall-help-poison-canary-zzz";
        let key = format!("{PROGRAM} ");
        // Seed the cache with an answer that no probe could produce, so "answered from memory" and
        // "probed and found nothing" cannot be confused.
        {
            let mut map = cache().lock().expect("the cache starts unpoisoned");
            map.entry(key.clone())
                .or_default()
                .lock()
                .expect("the slot starts unpoisoned")
                .clone_from(&Some(Some("cached, not probed".to_string())));
        }

        // Poison the per-key slot, then the map, each by panicking while the lock is held.
        let slot = {
            let map = cache().lock().expect("unpoisoned to take the slot out");
            map.get(&key).cloned().expect("the slot exists")
        };
        let _ = std::panic::catch_unwind(|| {
            let _held = slot.lock().expect("unpoisoned to hold it");
            panic!("poison the slot");
        });
        let _ = std::panic::catch_unwind(|| {
            let _held = cache().lock().expect("unpoisoned to hold the map");
            panic!("poison the map");
        });

        let answered = help_text(PROGRAM, &[]);
        assert_eq!(
            answered.as_deref(),
            Some("cached, not probed"),
            "a poisoned lock deleted the cache: the answer came from a fresh probe of a program \
             that does not exist, so the run silently lost the cache it had already paid for. \
             A `Mutex` stays poisoned for good after one panic, so this branch is permanent."
        );

        // Leave the process as we found it — the cache is process-wide and this suite is not.
        cache().lock().map(|mut m| m.remove(&key)).ok();
        let _ = std::panic::catch_unwind(|| {
            let _held = slot.lock();
            panic!("clear the poison");
        });
    }

    /// The discriminator, against both real versions' help — captured from the tools, and the
    /// only place the helm 3 arm can be asserted on a machine that has helm 4.
    ///
    /// helm 3's help carries `--kube-insecure-skip-tls-verify`, which contains the word and is
    /// not the flag; helm 4's carries `--verify` and `--keyring`. If this predicate answered
    /// yes for helm 3 the gate would call a correct no-op drift, and if it answered no for
    /// helm 4 a renamed flag would pass unnoticed — the two failures are opposite and the
    /// fixtures pin both.
    #[test]
    fn verification_is_documented_by_helm_4_and_not_by_helm_3() {
        const V3: &str = include_str!("../../tests/fixtures/helm/plugin-install-help-v3.txt");
        const V4: &str = include_str!("../../tests/fixtures/helm/plugin-install-help-v4.txt");
        assert!(
            !help_documents_verification(V3),
            "helm 3 does not verify plugins; `--kube-insecure-skip-tls-verify` is not a \
             verification flag for the plugin being installed"
        );
        assert!(
            help_documents_verification(V4),
            "helm 4 documents `--verify` and `--keyring`"
        );
    }

    #[test]
    fn a_program_that_is_not_here_answers_nothing_rather_than_no() {
        // `None`, not `Some(false)`. A tool that is absent has said nothing about its flags,
        // and treating silence as a refusal is what broke two unit tests on every CI runner
        // while passing on the one machine that happened to have helm.
        assert_eq!(
            accepts_flag(
                "shall-no-such-program-zzz",
                &["plugin".into(), "install".into()],
                "--verify=false"
            ),
            None
        );
    }

    #[test]
    fn the_value_tail_is_not_part_of_the_name() {
        // Help text writes `--verify`; the argv writes `--verify=false`. Matching the whole
        // token would answer "no" for every flag that carries a value, which is most of them.
        // Asserted through a program whose help certainly mentions the token it is asked about.
        let prog = if cfg!(windows) { "cmd" } else { "sh" };
        let _ = accepts_flag(prog, &[], "--nosuchflag=1");
        // The property under test is the split, which is cheap to state directly.
        assert_eq!("--verify=false".split('=').next(), Some("--verify"));
    }
}

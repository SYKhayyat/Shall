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
/// **`pub` for the integration tests' sake.** The cache mirror must poison and inspect the same
/// process-wide cache rather than reproducing its locking policy.
#[doc(hidden)]
pub fn cache() -> &'static Mutex<HashMap<String, HelpSlot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, HelpSlot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// What `program <chain…> --help` prints, or `None` when it could not be asked.
/// **`pub` for the integration tests' sake.** The cache mirror must exercise the real probe seam
/// after poisoning its locks.
#[doc(hidden)]
pub fn help_text(program: &str, chain: &[String]) -> Option<String> {
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
/// **`pub` for the integration tests' sake.** The flag-matching mirror must exercise this exact
/// boundary predicate rather than duplicate it.
#[doc(hidden)]
pub fn mentions_flag(text: &str, name: &str) -> bool {
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

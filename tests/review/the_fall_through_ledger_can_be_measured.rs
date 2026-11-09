// tests/review/the_fall_through_ledger_can_be_measured.rs
//
// **`#40`: an oracle that reports and never asserts.** `C4` wired the `unstubbed` ledger;
// `C23` made it *measurable*.
//
// The two claims are different and only the first was true. `deny_unstubbed()` is per-mock by
// design — 172 tests legitimately leave a command unanswered, and a global deny would redden a
// quarter of the suite for having no opinion — so the count of who does could only be found by
// adding the call one test at a time and watching which ones failed.
//
// `SHALL_AUDIT_UNSTUBBED=1` answers that. Three things about it are worth pinning, because each is
// a way it could silently stop being true.
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn executor() -> String {
    std::fs::read_to_string(repo().join("src/core/executor.rs")).expect("src/core/executor.rs")
}

/// **The audit arm is ahead of the early return that would hide the tests it measures.**
///
/// This is not a hypothetical. The first draft placed the arm inside
/// `assert_every_registration_was_used`, which opens with a return for a mock that allowed
/// unmatched registrations — and a test asserting only argv or state afterwards is exactly the
/// kind that allows them. It reported zero and looked like a clean bill of health.
///
/// The position is the assertion: the audit `println!` must appear before the first
/// `allow_unmatched` check in `drop`, and in `drop` rather than in the assertion helper.
#[test]
fn the_audit_reports_before_any_early_return_can_hide_it() {
    let body = executor();

    let drop_start = body
        .find("impl Drop for MockExecutor {")
        .expect("the Drop impl");
    let drop_body = &body[drop_start..];
    let drop_end = drop_body.find("\n}\n").expect("the Drop impl's close");
    let drop_body = &drop_body[..drop_end];

    let audit_at = drop_body
        .find("SHALL_AUDIT_UNSTUBBED")
        .expect("the audit arm is inside `drop`, where the ledger is complete");
    let assert_at = drop_body
        .find("assert_every_registration_was_used()")
        .expect("the unused-registration assertion is still called from `drop`");

    assert!(
        audit_at < assert_at,
        "the audit arm now runs after `assert_every_registration_was_used`, which returns early for \
         a mock that allowed unmatched registrations — so it would report nothing for exactly the \
         tests it exists to find"
    );

    // And it must not be *inside* that helper, which is where it started and where it reported zero.
    let helper_start = body
        .find("pub fn assert_every_registration_was_used(&self)")
        .expect("the assertion helper");
    let helper_body = &body[helper_start..];
    let helper_end = helper_body
        .find("\n    pub fn ")
        .or_else(|| helper_body.find("\n    /// "))
        .unwrap_or(helper_body.len());
    assert!(
        !helper_body[..helper_end].contains("SHALL_AUDIT_UNSTUBBED"),
        "the audit arm is back inside `assert_every_registration_was_used`, behind the \
         `allow_unmatched_registrations` early return that made it report zero"
    );
}

/// **The audit never fails a build.** It is a measurement, and `deny_unstubbed()` remains the only
/// thing that panics on a fall-through. An audit that could fail would be turned off the first time
/// it was inconvenient, which returns the ledger to unread.
#[test]
fn audit_mode_reports_and_never_panics() {
    let body = executor();

    let audit_at = body.find("SHALL_AUDIT_UNSTUBBED").expect("the audit arm");
    // The arm's own body: from the environment read to the closing brace of the `if`.
    let arm = &body[audit_at..];
    let arm_end = arm
        .find("assert_every_registration_was_used();")
        .unwrap_or(arm.len());
    let arm = &arm[..arm_end];

    assert!(
        !arm.contains("panic!") && !arm.contains("assert!"),
        "the audit arm now fails rather than reports. `deny_unstubbed()` is the opt-in that fails; \
         a global audit that reddens a quarter of the suite for having no opinion is the thing \
         `C23` refused"
    );
    assert!(
        arm.contains("println!"),
        "the audit arm no longer prints, so there is nothing to measure"
    );
}

/// **The fixture seeds every name the suite probes.**
///
/// `check_command` answers `true` for an unseeded name and records the probe, so a fixture seeding
/// only `brew`/`apt`/`cargo` left 56 names falling through on every test that asked. That is not a
/// defect — `true` is what an unopinionated test means — but it made the ledger unreadable: 134 of
/// 352 reporting tests reported *only* probes. Seeding them moved that to 10 and left the real
/// queries untouched, which is the measurement `#40`'s follow-up works from.
#[test]
fn the_fixture_seeds_the_managers_the_suite_probes() {
    let fixture = std::fs::read_to_string(repo().join("tests/mock_providers/mod.rs"))
        .expect("tests/mock_providers/mod.rs");

    let seeded = fixture.matches("set_command_exists(").count();
    assert!(
        seeded >= 50,
        "the fixture seeds {seeded} command names. It seeded 59 when `C23` was measured; a drop \
         back toward three re-unreadable-ises the fall-through ledger, because every probe the \
         fixture does not seed is recorded as an unanswered command and drowns the real ones"
    );

    // The three it always had, plus the ones the audit named as the loudest offenders.
    for name in [
        "brew",
        "apt",
        "cargo",
        "btrfs",
        "timeshift",
        "pacman",
        "zfs",
        "flatpak",
    ] {
        assert!(
            fixture.contains(&format!("set_command_exists(\"{name}\", true)")),
            "`{name}` is probed by the suite and is no longer seeded, so every test that asks \
             about it reports an unanswered command that is not a defect"
        );
    }
}

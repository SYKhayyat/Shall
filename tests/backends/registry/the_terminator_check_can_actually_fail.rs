use super::*;

/// The terminator check, run against argv this test writes rather than argv the tree
/// produces.
///
/// **The gate above passes today, and that is exactly when an instrument stops being
/// evidence.** It fired on `dnf install -y jq` and `pacman -S --noconfirm --needed jq` on
/// the day it was written; nothing since then would notice if it quietly stopped seeing
/// anything, because a scan that matches nothing and a tree with nothing to match look the
/// same from the outside. So the shapes are asserted here — the two it must catch, and the
/// four it must not — before it is trusted with the real table.
#[test]
fn the_terminator_check_can_actually_fail() {
    fn probe(backend: &'static str, subject: &'static str, call: &str) -> usize {
        let case = ArgvCase::shaped(
            backend,
            &|_, _| {},
            subject,
            &[],
            Expect::NoCommand("unused — this drives the scan, not a backend"),
            Expect::NoCommand("unused — this drives the scan, not a backend"),
        );
        operands_outside_the_terminator(&case, &[call.to_string()]).len()
    }

    // Caught: a terminating program handed the declaration's own text bare. These are the
    // two real invocations that were live when this was written.
    // `dnf` is no longer in the terminating set (it may be dnf5), so its bare operand is
    // correct and the scan must NOT flag it. `apt` is the terminating system manager this
    // half of the check still has to catch.
    assert_eq!(probe("dnf", "jq", "dnf install -y jq"), 0);
    assert_eq!(probe("apt-get", "jq", "apt-get install -y jq"), 1);
    assert_eq!(
        probe("pacman", "jq", "pacman -S --noconfirm --needed jq"),
        1
    );
    // And `winget`, which joined the terminating set on 2026-08-11 when the differential
    // probe measured it on windows-latest — it had been listed as not terminating on the
    // shape of its parser, and it was one of three rows that reasoning got wrong. It sat on
    // the "not caught" side of this self-test on the strength of that listing, which is the
    // instrument agreeing with the claim it was meant to check.
    assert_eq!(probe("winget", "jq", "winget install jq"), 1);

    // Not caught, and each for its own reason — a scan that flagged these would be turned
    // off within a week.
    assert_eq!(probe("apt", "jq", "apt install -y -- jq"), 0, "terminated");
    assert_eq!(
        probe("nix", "jq", "nix profile install -- nixpkgs#jq"),
        0,
        "terminated, and the operand only contains the subject"
    );
    assert_eq!(
        probe("scoop", "jq", "scoop install jq"),
        0,
        "scoop is a PowerShell script dispatching on $args[0]; a bare `--` becomes an app \
         name, so a bare operand is the correct argv and must not be flagged"
    );
    assert_eq!(
        probe("apt", "jq", "apt-get autoremove --dry-run"),
        0,
        "no operand carries the subject"
    );
    assert_eq!(
        probe("emacs", "jq", "emacs --batch --eval (package-install 'jq)"),
        0,
        "written exemption: the form is the value of `--eval`, not an operand"
    );
}

#!/usr/bin/env bash
# ============================================================================
# The integration harnesses' own predicates, tested.
#
# The harnesses run nightly and in containers, so a wrong verdict inside one of
# them is found a day late and by whoever reads the log — which is how `go` was
# reported as leaving a binary behind on a removal that had worked, and how a
# macOS sweep in which nothing executed at all reported passes.
#
# Both faults were in two-line predicates. This runs those predicates against the
# cases that got them wrong, in a second, on every push.
#
# The function bodies are LIFTED from the harnesses rather than copied, so this
# cannot drift from what CI actually runs.
#
#   scripts/harness-logic-test.sh [harness.sh ...]
# ============================================================================
set -u

SOURCES="$*"
if [ -z "$SOURCES" ]; then
    _here="$(cd "$(dirname "$0")/.." && pwd)"
    SOURCES="$_here/docker/integration/run-in-container.sh $_here/scripts/integration-windows.sh"
fi

TOTAL=0; BAD=0

# Pull one function's definition out of a harness, so this file tests what CI runs rather
# than a copy of it. Handles both the one-line `f() { …; }` form and the braces-on-their-
# own-line form, and refuses anything implausibly long: a range that runs off the end of a
# function swallows the rest of the script, and `eval` would then RUN it — which is how a
# first attempt at this printed the harness's banner and died on an unset variable.
lift() {
    _name="$1"; _file="$2"
    awk -v fn="$_name" '
        $0 ~ "^" fn "\\(\\) \\{" { inside = 1 }
        inside { print; n++ }
        inside && /\}[[:space:]]*$/ && (n > 1 || /\{.*\}/) { exit }
        n > 60 { exit 1 }
    ' "$_file"
}
# The cap is a runaway guard — a malformed function must not slurp the rest of the file — and
# not a size limit on harness functions. It was 40, and `classify_install` grew to 43 when it
# stopped re-deriving transience by retrying and started reading `shall-failure-class:`. A
# truncated lift is a `syntax error: unexpected end of file` and then `CLASS: unbound variable`,
# which reads as the harness being broken rather than this file's awk being short: worth
# knowing, because the first instinct was to shrink the function to fit the test measuring it.

outcome() { # label want cmd...
    _olabel="$1"; _owant="$2"; shift 2
    TOTAL=$((TOTAL + 1))
    _op0=$PASS; _of0=$FAILC; _os0=$SOFTC
    "$@" >/dev/null 2>&1
    if   [ "$PASS"  -gt "$_op0" ]; then _ogot=pass
    elif [ "$FAILC" -gt "$_of0" ]; then _ogot=fail
    elif [ "$SOFTC" -gt "$_os0" ]; then _ogot=soft
    else _ogot=none; fi
    if [ "$_ogot" = "$_owant" ]; then
        echo "  ok    $_olabel -> $_ogot"
    else
        echo "  BAD   $_olabel -> $_ogot (wanted $_owant)"; BAD=$((BAD + 1))
    fi
}

run_against() {
    SRC="$1"
    echo "== $SRC"
    [ -f "$SRC" ] || { echo "  FATAL: no such harness"; BAD=$((BAD + 1)); return 1; }

    PASS=0; FAILC=0; SOFTC=0; FAILED_NAMES=""
    soft() { SOFTC=$((SOFTC + 1)); }
    removal_leaves_binary() { case "$1" in bun) echo "bun keeps its launcher" ;; *) echo "" ;; esac; }

    FAKE=""; FAKE_NAME="hello"
    path_of() { case "$1" in "$FAKE_NAME") echo "$FAKE" ;; *) echo "" ;; esac; }

    for _fn in never_ran assert_binary_gone on_path named_bin_dir off_path_copy binary_present assert_binary_reachable; do
        _body="$(lift "$_fn" "$SRC")"
        if [ -z "$_body" ]; then
            echo "  FATAL: could not lift $_fn() from this harness"
            TOTAL=$((TOTAL + 1)); BAD=$((BAD + 1)); return 1
        fi
        eval "$_body"
    done
    command -v assert_binary_gone >/dev/null || { echo "  FATAL: could not lift assert_binary_gone"; BAD=$((BAD + 1)); return 1; }
    command -v never_ran >/dev/null || { echo "  FATAL: could not lift never_ran"; BAD=$((BAD + 1)); return 1; }

    check() { # check <label> <pass|fail|soft>
        _label="$1"; _want="$2"
        TOTAL=$((TOTAL + 1))
        _p0=$PASS; _f0=$FAILC; _s0=$SOFTC
        assert_binary_gone "$_be" "$_bin" "$_was" >/dev/null 2>&1
        if   [ "$PASS"  -gt "$_p0" ]; then _got=pass
        elif [ "$FAILC" -gt "$_f0" ]; then _got=fail
        elif [ "$SOFTC" -gt "$_s0" ]; then _got=soft
        else _got=none; fi
        if [ "$_got" = "$_want" ]; then
            echo "  ok    $_label -> $_got"
        else
            echo "  BAD   $_label -> $_got (wanted $_want)"; BAD=$((BAD + 1))
        fi
    }

    # The regression: cabal's canary binary is `hello` and cabal has no uninstall verb,
    # so its copy is there before go installs its own and still there after go removes
    # it. Asking PATH scored another manager's deliberate leftover as go's failure.
    _be=go; _bin=hello; _was="/root/.cabal/bin/hello"; FAKE="/root/.cabal/bin/hello"
    check "another manager's copy predates and outlives the install" pass

    # The defect the check exists to catch, which must still be caught.
    _be=go; _bin=hello; _was=""; FAKE="/root/go/bin/hello"
    check "removal left its own binary behind" fail

    _be=go; _bin=hello; _was=""; FAKE=""
    check "clean removal" pass

    # A documented leftover softens only when it actually happens.
    _be=bun; _bin=hello; _was=""; FAKE="/root/.bun/bin/hello"
    check "documented leftover softens" soft
    _be=bun; _bin=hello; _was=""; FAKE=""
    check "documented quirk that did not happen still passes strictly" pass

    # Resolution that MOVED to this backend's own directory is its leftover, not
    # the pre-existing copy.
    _be=go; _bin=hello; _was="/root/.cabal/bin/hello"; FAKE="/root/go/bin/hello"
    check "resolution moved to this backend's own dir" fail

    # A command that could not run is not a refusal.
    for rc in 127 126 124; do
        TOTAL=$((TOTAL + 1))
        if never_ran "$rc"; then echo "  ok    rc=$rc counts as never-ran"
        else echo "  BAD   rc=$rc must count as never-ran"; BAD=$((BAD + 1)); fi
    done
    for rc in 0 1 2 3; do
        TOTAL=$((TOTAL + 1))
        if never_ran "$rc"; then echo "  BAD   rc=$rc must be a real verdict"; BAD=$((BAD + 1))
        else echo "  ok    rc=$rc is a real verdict"; fi
    done

    # A predicate answers yes or no. `command -v` says "not found" with 1 under bash and
    # 127 under dash and busybox ash — and 127 is what `nok` must read as "never ran", so
    # an unnormalised on_path made three container jobs fail on a name that was correctly
    # absent. This case only bites where /bin/sh is dash: ubuntu's runner, which is where
    # this file runs.
    TOTAL=$((TOTAL + 1))
    on_path shall-no-such-binary-zzz; _rc=$?
    if [ "$_rc" = 1 ]; then
        echo "  ok    on_path says no with 1, not with the shell's not-found code"
    else
        echo "  BAD   on_path answered 'no' with rc=$_rc — nok cannot tell that from 'never ran'"
        BAD=$((BAD + 1))
    fi
    TOTAL=$((TOTAL + 1))
    if on_path sh; then echo "  ok    on_path still finds a name that is there"
    else echo "  BAD   on_path could not find sh"; BAD=$((BAD + 1)); fi

    # ---- E6c/W4: "the binary is reachable" is a claim about the PRODUCT ------
    #
    # `on_path` alone asks the HOST. On a clean runner ~/go/bin, ~/.local/bin and yarn's
    # global directory are on nobody's PATH, so the old check failed three backends that
    # had done nothing wrong — and it would have passed all three had the machine happened
    # to be wired, which is the half that matters: an install that says NOTHING about an
    # unreachable binary is the defect (github and yarn, measured 2026-07-29).
    _rd="${TMPDIR:-/tmp}/shall-reach-$$"
    rm -rf "$_rd"; mkdir -p "$_rd"
    _rbin=shall-reach-zzz
    _rlog="$_rd/install.log"
    printf '%s\n' "  WARN shall::verbs::sync: \`go\` installs its executables into $_rd, which is not on your PATH — so what it just installed will answer \"command not found\"." > "$_rlog"

    # The defect this check exists for: installed, unreachable, and nothing said so.
    outcome "unreachable and unexplained" fail assert_binary_reachable go "$_rbin" "$_rd/no-such.log"

    # Shall warned and the file is where it said: the product kept its promise, and the
    # host's PATH is not the product's to fix.
    : > "$_rd/$_rbin"
    outcome "unreachable, explained, and there" pass assert_binary_reachable go "$_rbin" "$_rlog"

    # Warned about a directory the binary is NOT in — the install claimed something untrue.
    rm -f "$_rd/$_rbin"
    outcome "explained and not there" fail assert_binary_reachable go "$_rbin" "$_rlog"

    # A Windows install writes cowsay.cmd, not cowsay. Looking only for the bare name reports
    # an installed program as missing.
    : > "$_rd/$_rbin.cmd"
    outcome "the platform's extension counts" pass assert_binary_reachable go "$_rbin" "$_rlog"
    rm -f "$_rd/$_rbin.cmd"

    # G-3, the reachability half of the collision `assert_binary_gone` has handled since
    # 2026-07-29. Measured on the tools image (CI 30566924407): `PASS go: hello is on PATH`
    # scored against /root/.cabal/bin/hello, which cabal installed four lifecycles earlier and
    # cannot uninstall. The check would have passed if the go install had done nothing at all.
    FAKE="/root/.cabal/bin/hello"
    outcome "the name resolves to the manager that already owned it" fail \
        assert_binary_reachable go hello "$_rd/no-such.log" "/root/.cabal/bin/hello"

    # Same collision, but this backend did install its own copy and said where it went. The
    # other manager holding the PATH entry is not this install's failure.
    : > "$_rd/hello"
    outcome "a collision the backend's own copy answers for" pass \
        assert_binary_reachable go hello "$_rlog" "/root/.cabal/bin/hello"
    rm -f "$_rd/hello"

    # The control, and it is the whole reason the comparison is against the PRIOR value rather
    # than against a list of known collisions: a name that resolves somewhere it did not before
    # is this install's doing, and must still pass with nothing else to go on.
    FAKE="/root/go/bin/hello"
    outcome "a resolution that changed is this install's" pass \
        assert_binary_reachable go hello "$_rd/no-such.log" "/root/.cabal/bin/hello"
    # The lifted bodies assign to globals — there are no locals in a POSIX shell — so the three
    # cases above left `$_rbin` reading `hello`. Restored, because the checks below share it.
    FAKE=""; _rbin=shall-reach-zzz

    # The warning belongs to the backend that printed it. One sync can warn about two
    # managers, and handing yarn's directory to go would answer for the wrong install.
    TOTAL=$((TOTAL + 1))
    if [ -z "$(named_bin_dir yarn "$_rlog")" ]; then
        echo "  ok    a directory is read only for the backend that named it"
    else
        echo "  BAD   named_bin_dir handed go's directory to yarn"; BAD=$((BAD + 1))
    fi

    # And the removal half. A binary that was never on PATH is "gone" by PATH before the
    # removal runs, so the old three-argument check passed while the file was still there.
    : > "$_rd/$_rbin"
    _be=go; _bin="$_rbin"; _was=""; FAKE=""
    TOTAL=$((TOTAL + 1))
    _p0=$PASS; _f0=$FAILC
    assert_binary_gone go "$_rbin" "" "$_rlog" >/dev/null 2>&1
    if [ "$FAILC" -gt "$_f0" ]; then
        echo "  ok    a leftover off PATH is still a leftover -> fail"
    else
        echo "  BAD   a leftover in the directory the install named passed as removed"; BAD=$((BAD + 1))
    fi
    rm -f "$_rd/$_rbin"
    TOTAL=$((TOTAL + 1))
    _f0=$FAILC
    assert_binary_gone go "$_rbin" "" "$_rlog" >/dev/null 2>&1
    if [ "$FAILC" -eq "$_f0" ]; then
        echo "  ok    a removal that really removed it still passes"
    else
        echo "  BAD   a clean removal was scored as a leftover"; BAD=$((BAD + 1))
    fi
    rm -rf "$_rd"
}

for src in $SOURCES; do run_against "$src"; done

# ---------------------------------------------------------------------------
# Every `gone_ok` tag must be recorded by a `witness`, in the same harness.
#
# **`gone_ok` refuses to score an absence for a subject nothing was ever seen leaving behind** —
# that is the control, and it is the whole reason the instrument works. The cost of a control is
# that its own absence is a failure, and that failure reads exactly like a defect in the product
# under test. Which is what happened: `gone_ok "the dir is gone from disk" dir-dst` had no
# `witness dir-dst` anywhere, so the check could never be scored, and **eight distro legs went
# red with `the declared dir is on disk` and `the declared dir has its mode` both PASSING.** The
# harness was reporting a missing line of itself in the vocabulary of a broken product, which is
# the worst of both faults at once.
#
# So this is a scan over the harness **as text**, with a floor: a scan that matched nothing would
# pass, and a gate that cannot fail is what II.23 is about. The floor is the tag count, and the
# `link:` block's own witness is the control — a harness where the pair exists must not be
# reported, which is what stops this from flagging every `gone_ok` in existence.
#
# ## every_gone_ok_tag_is_witnessed_somewhere
#
# **The name is here because three documents cite it and it existed in none of them.**
# `run-in-container.sh`'s `dir:` block names this gate as what stops the next twin being added
# without a witness, and `why.md` and `spec/plan.md` both name it too — all three written while
# this was an inline block with an `echo` header, so a reader who grepped the citation found prose
# and no gate. That is the same defect this repository files issues about for `V.n`
# ("a citation that resolves to nothing makes the mandatory read impossible"), in the one corpus
# that has no test for it yet: **`V.n` citations are checked by
# `every_citation_of_a_why_entry_resolves_to_one`, and gate names are not checked at all.**
#
# It is a block rather than a function because it mutates this file's shared `TOTAL`/`BAD`
# counters, and wrapping it would be a refactor of a working gate to serve a comment. The marker
# is the smaller change that makes the three citations true, and `grep` now resolves them.
for src in $SOURCES; do
    echo "== $src: every gone_ok tag is witnessed in the same harness"
    if [ ! -f "$src" ]; then echo "  FATAL: no such harness"; BAD=$((BAD + 1)); continue; fi

    _gt="$(grep -oE 'gone_ok "[^"]*" [a-z][a-z0-9-]*' "$src" | grep -oE '[a-z][a-z0-9-]*$' | sort -u)"
    _wt="$(grep -oE '^[[:space:]]*witness [a-z][a-z0-9-]*' "$src" | grep -oE '[a-z][a-z0-9-]*$' | sort -u)"

    TOTAL=$((TOTAL + 1))
    _gn=$(printf '%s\n' "$_gt" | grep -c . )
    _wn=$(printf '%s\n' "$_wt" | grep -c . )
    # The floor first: on a tree where the scan stopped matching, the answer below would be
    # "nothing is missing", which is the passing answer for a scan that examined nothing.
    # A floor of 2, and it is 2 because the Windows harness genuinely has two: `pkg-binary` and
    # `registry-value`, both witnessed. A floor set to the container harness's ten would report
    # that harness as broken, which is the move this file keeps refusing — a budget read off one
    # host class enforced on another. Two is the floor that still catches "the scan stopped
    # matching", which is the failure it exists for; what discriminates a witnessed tag from an
    # unwitnessed one is the control below, not the count.
    if [ "$_gn" -lt 2 ] || [ "$_wn" -lt 2 ]; then
        echo "  BAD   found $_gn gone_ok tag(s) and $_wn witness(es); the scan has stopped"
        echo "        matching the file, and \"nothing is unwitnessed\" would be a pass"
        BAD=$((BAD + 1))
        continue
    fi

    _missing="$(comm -23 <(printf '%s\n' "$_gt") <(printf '%s\n' "$_wt"))"
    if [ -n "$_missing" ]; then
        echo "  BAD   gone_ok tag(s) nothing ever witnesses:"
        printf '        %s\n' $_missing
        echo "        Each of these can never be scored, so its harness reports the missing"
        echo "        witness as if the product had failed to create the thing. Add the"
        echo "        \`witness <tag>\` at the site that asserts the presence, or drop the"
        echo "        gone_ok. Measured: \`dir-dst\` did this on all eight distro legs."
        BAD=$((BAD + 1))
    else
        echo "  ok    all $_gn gone_ok tag(s) have a witness in the same harness"
    fi

    # The control: a tag that IS witnessed must not be reported, and the shape of the answer
    # above must be the passing one when there is nothing missing.
    # The control, and it uses EACH harness's own vocabulary rather than a shared tag: the point
    # is that a tag present on both sides is not reported, so the check can tell a witnessed tag
    # from an unwitnessed one rather than only ever answering "nothing is missing".
    _ctrl="$(printf '%s\n' "$_gt" | head -1)"
    TOTAL=$((TOTAL + 1))
    if [ -n "$_ctrl" ] && printf '%s\n' "$_wt" | grep -qx "$_ctrl"; then
        echo "  ok    \`$_ctrl\` is on both sides and not reported — the scan discriminates"
    else
        echo "  BAD   the control is wrong: \"$_ctrl\" is not on both sides, so this check"
        echo "        cannot tell a witnessed tag from an unwitnessed one"
        BAD=$((BAD + 1))
    fi
done

# ---------------------------------------------------------------------------
# `witness`, `gone_ok` and `stays_ok`, driven in both directions.
#
# II.23: the oracle drives the same predicate the check drives. These three exist to make a
# harness check capable of failing, and the only way to know one still can is to take it away
# from a working run and watch it go red — a check of the check that stops matching is a check
# that stops being a check.
#
# **`stays_ok` is the newest and the one that needed this most**, because the defect it fixes is
# invisible to a review of the harness text: `ok "$PKG is still installed after unmanage"` reads
# as an assertion, and on a machine that already had the package it was one — just not of
# Shall. Both directions are driven, and so is the one that matters most, which is the one where
# the subject is present and unwitnessed. That case is the whole defect, and if the predicate
# ever stops refusing it, every survival proof in both harnesses silently returns to being a
# tautology.
for src in $SOURCES; do
    echo "== $src: a survival proof needs an arrival"
    [ -f "$src" ] || { echo "  FATAL: no such harness"; BAD=$((BAD + 1)); continue; }

    PASS=0; FAILC=0; SOFTC=0; FAILED_NAMES=""
    soft() { SOFTC=$((SOFTC + 1)); }
    hard() { FAILC=$((FAILC + 1)); FAILED_NAMES="$FAILED_NAMES
    - $1"; }

    for _fn in _seen_tag witness gone_ok stays_ok; do
        _body="$(lift "$_fn" "$src")"
        if [ -z "$_body" ]; then
            echo "  FATAL: could not lift $_fn() from $src"
            TOTAL=$((TOTAL + 1)); BAD=$((BAD + 1)); continue 2
        fi
        eval "$_body"
    done

    _wd="$(mktemp -d)"; LEDGER="$_wd"; mkdir -p "$LEDGER"
    : > "$_wd/present"; rm -f "$_wd/absent"

    # The control, and the reason the other three are worth reading: a survival proof about a
    # package this run installed. `witness` on something true, then the command passes.
    witness pkg test -f "$_wd/present"
    outcome "a subject that arrived and is still there" pass \
        stays_ok "arrived and survived" pkg test -f "$_wd/present"
    # **The defect.** The subject is right there and this run never saw it arrive — the machine
    # had it before the harness started. A bare `ok` scores this `pass`, on a `shall` that
    # installed nothing, which is the entire reason the predicate exists.
    outcome "a subject this run never saw arrive" fail \
        stays_ok "present but unwitnessed" never-arrived test -f "$_wd/present"
    # And the command actually failing is not a substitute for the arrival — that is how a
    # package the run removed for its own reasons would pass as a survival proof.
    outcome "an arrival recorded and the subject then gone" fail \
        stays_ok "arrived and vanished" pkg test -f "$_wd/absent"

    # The other direction, because the two are the same rule read from opposite ends and an
    # oracle that only drives one of them is half an oracle.
    outcome "an arrival that went away" pass \
        gone_ok "arrived and removed" pkg test -f "$_wd/absent"
    outcome "an absence with no arrival behind it" fail \
        gone_ok "gone but never seen" never-arrived test -f "$_wd/absent"

    rm -rf "$_wd"
done

# ---------------------------------------------------------------------------
# Every subcommand a harness invokes must exist in the binary.
#
# This is not pedantry: the `doctor`, `status`, `absent`, `unmanaged`, `conflicts` and
# `audit` were folded into `check <section>`, the host harness was never updated, and
# clap answered "unrecognized subcommand" with exit 2. One of those calls builds
# READY_LIST — so the entire real-lifecycle section and the entire plan-smoke section
# iterated over an empty list and reported nothing wrong. A stale name does not announce
# itself as missing coverage; it announces itself as no coverage at all.
#
# Runs only when a binary is given, so the predicate tests above stay runnable anywhere.
BIN="${SHALL_BIN:-}"
if [ -n "$BIN" ] && "$BIN" --version >/dev/null 2>&1; then
    echo "== subcommands invoked vs subcommands that exist ($BIN)"
    _real="$("$BIN" --help 2>&1 | sed -n '/^Commands:/,/^Options:/p' \
        | awk '{print $1}' | grep -E '^[a-z]' | sort -u)"
    for src in $SOURCES; do
        _used="$(grep -oE '(\b(lx|lx_slow|smoke_lx|silent_lx|restore_lx))( +-[-a-zA-Z]+)* +[a-z][a-z-]*' "$src" \
            | awk '{print $NF}' | sort -u)"
        _unknown=""
        for _v in $_used; do
            printf '%s\n' "$_real" | grep -qx "$_v" || _unknown="$_unknown $_v"
        done
        TOTAL=$((TOTAL + 1))
        if [ -z "$_unknown" ]; then
            echo "  ok    $(basename "$src") invokes only subcommands that exist"
        else
            echo "  BAD   $(basename "$src") invokes subcommands the binary does not have:$_unknown"
            BAD=$((BAD + 1))
        fi
    done
    # The other half, and the half that was missing. An exemption says "this subcommand
    # exists and cannot be driven here, for this reason" — so a name that does not exist
    # cannot be exempt, it can only be stale. `undo` sat in both lists after the command was
    # renamed to snapshot/rollback, and nothing looked, because the audit only ever checked
    # the names that were *used*. An unvalidated exemption list is where coverage goes to
    # disappear quietly: the printed "5 exempt" was wrong and read as reassurance.
    echo "== subcommands exempted vs subcommands that exist ($BIN)"
    for src in $SOURCES; do
        _exempt="$(sed -n 's/^[[:space:]]*EXEMPT_CMDS="\([^"]*\)".*/\1/p' "$src" | tr ' ' '\n' \
            | grep -E '^[a-z]' | sort -u)"
        [ -n "$_exempt" ] || continue
        _stale=""
        for _v in $_exempt; do
            printf '%s\n' "$_real" | grep -qx "$_v" || _stale="$_stale $_v"
        done
        TOTAL=$((TOTAL + 1))
        if [ -z "$_stale" ]; then
            echo "  ok    $(basename "$src") exempts only subcommands that exist"
        else
            echo "  BAD   $(basename "$src") exempts subcommands the binary does not have:$_stale"
            BAD=$((BAD + 1))
        fi
    done
else
    echo "== subcommands invoked vs subcommands that exist: SKIPPED (set SHALL_BIN to a built binary)"
fi

# E5's classifier, in both harnesses. The catch-all it replaced softened ANY install failure
# to "network/ecosystem variance" and skipped that backend's whole remaining lifecycle; in one
# observed run it fired four times and not once was it the network. The verdicts are what
# matter, so they are tested rather than the wording: a refusal is a pass, a timeout is a soft,
# and — the half that actually lost coverage — a transient must let the lifecycle CONTINUE.
echo "== an install failure is classified, not assumed to be the network"
for _src in $SOURCES; do
    TOTAL=$((TOTAL + 1))
    _body="$(lift classify_install "$_src")"
    if [ -z "$_body" ]; then
        echo "  BAD   $(basename "$_src") has no classify_install(): its install failures are unclassified"
        BAD=$((BAD + 1))
        continue
    fi
    (
        PASS=0; FAILC=0; SOFTC=0
        # The environment the lifted body runs in, not this script's own variables — both are
        # read by `run-in-container.sh`'s `classify_install` and its callers, and shellcheck
        # cannot see through the `eval` below to know that. Dropping them would make the lifted
        # code run against unset names, which is the failure this whole file exists to catch.
        #
        # One directive per line, and the assignments split off the line above for that reason:
        # a directive attaches to the next *command*, so on `A=1; B=2` it covers `A` and leaves
        # `B` reported — which is how the first attempt at this suppression did nothing.
        # shellcheck disable=SC2034
        FAILED_NAMES=""
        # shellcheck disable=SC2034
        TO_LONG="timeout 900"
        soft() { SOFTC=$((SOFTC + 1)); }
        hard() { FAILC=$((FAILC + 1)); }
        refused() { PASS=$((PASS + 1)); }
        eval "$_body"
        _log="$(mktemp)"; : > "$_log"

        _bad=0
        classify_install be spec 3 "$_log" >/dev/null 2>&1
        [ "$CLASS" = refused ] && [ "$PASS" -eq 1 ] || { echo "  BAD   exit 3 is not scored as a refusal (got '$CLASS')"; _bad=1; }
        classify_install be spec 124 "$_log" >/dev/null 2>&1
        [ "$CLASS" = timeout ] && [ "$SOFTC" -eq 1 ] || { echo "  BAD   exit 124 is not scored as a timeout (got '$CLASS')"; _bad=1; }
        # Neither may be counted as a hard failure: that is the "refusal reported as a defect"
        # half of E5, and it is as wrong as the variance catch-all was.
        [ "$FAILC" -eq 0 ] || { echo "  BAD   a refusal or a timeout was recorded as a hard failure"; _bad=1; }

        # R-3, both directions. The classifier reads `shall-failure-class:` instead of retrying
        # the install to guess at it, and the two branches fail in opposite ways: a permanent
        # failure retried is a minute wasted per backend, and a transient one scored a defect is
        # a red CI leg over a rate-limit window that has since moved.
        LEDGER="$(mktemp -d)"; : > "$LEDGER/be-life-unmeasured"
        lx() { echo "the retry must not be reached"; return 1; }
        lx() { echo "the retry must not be reached"; return 1; }
        lx_slow() { lx "$@"; }

        FAILC=0; SOFTC=0; PASS=0
        printf 'shall-failure-class: permanent\n' > "$_log"
        # To a file, not a `$( )`: command substitution runs in a subshell, so `CLASS` set
        # inside it never reaches this scope and the assertion below reads the PREVIOUS call's
        # answer. It did, and reported `timeout`.
        _out="$(mktemp)"
        classify_install be spec 1 "$_log" > "$_out" 2>&1
        [ "$CLASS" = defect ] || { echo "  BAD   a permanent failure is not a defect (got '$CLASS')"; _bad=1; }
        grep -q retrying "$_out" && { echo "  BAD   a permanent failure was retried anyway"; _bad=1; }
        rm -f "$_out"

        FAILC=0; SOFTC=0
        printf 'shall-failure-class: transient\n' > "$_log"
        classify_install be spec 1 "$_log" >/dev/null 2>&1
        [ "$CLASS" = exhausted ] || { echo "  BAD   a transient failure that did not clear is not exhausted (got '$CLASS')"; _bad=1; }
        [ "$FAILC" -eq 0 ] || { echo "  BAD   a transient failure that did not clear was scored a hard failure — this is the red macOS leg"; _bad=1; }
        # **Which log the rest of the lifecycle reads.** A retry that cleared wrote its own
        # output, and every assertion after this one asks an install log where the binary went -
        # so a transient retry left them reading the attempt that failed. Measured on the guix
        # nightly of 2026-08-21: the install worked and the PATH check reported that nothing said
        # where it went, because it was looking at the wrong page.
        FAILC=0; SOFTC=0
        printf 'shall-failure-class: transient
' > "$_log"
        # BOTH names: the container harness retries through `lx_slow`, the Windows one through
        # `lx`, and a stub for one of them tests one twin and lies about the other.
        lx() { return 0; }
        lx_slow() { return 0; }
        classify_install be spec 1 "$_log" >/dev/null 2>&1
        [ "$CLASS" = transient ] || { echo "  BAD   a retry that cleared is not transient (got '$CLASS')"; _bad=1; }
        case "$LIFELOG" in
            */life2.out|*/itw-retry.out) : ;;
            *) echo "  BAD   after a cleared retry the lifecycle still reads '$LIFELOG', which is the attempt that failed"; _bad=1 ;;
        esac
        lx_slow() { lx "$@"; }

        grep -qx be "$LEDGER/be-life-unmeasured" || { echo "  BAD   an unmeasurable lifecycle was not recorded, so the ratchet cannot excuse it by name"; _bad=1; }

        # And a failure with no class at all is a defect, not a free pass: its absence means the
        # binary under test predates the line, so the run is not measuring the tree it claims to.
        FAILC=0
        : > "$_log"
        classify_install be spec 1 "$_log" >/dev/null 2>&1
        [ "$CLASS" = defect ] && [ "$FAILC" -eq 1 ] || { echo "  BAD   a failure with no class line was not scored a defect (got '$CLASS')"; _bad=1; }

        rm -rf "$LEDGER"
        rm -f "$_log"
        exit "$_bad"
    )
    if [ $? -eq 0 ]; then
        echo "  ok    $(basename "$_src") tells a refusal, a timeout, a defect and an unmeasurable apart"
    else
        BAD=$((BAD + 1))
    fi
done

# `install_delivered` — the gate #100 is about, driven in both directions.
#
# **`classify_install` cannot catch this, and the reason is the whole finding:** it only runs when
# the install *failed*, so an install that returned 0 having delivered nothing never reaches it.
# On 2026-10-02 `void` reported `PASS uv installed pyjokes for real` for four backends whose
# package was never on the machine, and every removal check below it passed *vacuously* —
# `nok "... is gone from list"` is satisfied by a grep that finds nothing.
# **One harness, and that is a fact rather than a loop that forgot to spread.** The Windows
# harness has no `install_delivered`: it drives its managers through `lx`, has no real-lifecycle
# section and no `be-life` ledger, so there is nothing for this gate to guard there. A loop over
# both would report "ok" for a harness the gate was never in.
for _src in $SOURCES; do
    # shellcheck disable=SC2043
    _body="$(lift install_delivered "$_src")"
    [ -n "$_body" ] || continue          # this harness does not have that lifecycle
    (
        # shellcheck disable=SC2034
        PASS=0; FAILC=0; SOFTC=0
        soft() { SOFTC=$((SOFTC + 1)); }
        # Both stubs are lifted too: the point is that `install_delivered` asks them, and a stub
        # here that the real body does not call would make this test pass for the wrong reason.
        _ok="$(lift grep_ok "$_src")"
        _abr="$(lift assert_binary_reachable "$_src")"
        _lcs="$(lift list_cannot_show "$_src")"
        # **The body under test is eval'd here, not just its stubs.** The first version of this
        # lifted the three helpers and forgot the gate, so every case reported
        # `install_delivered: command not found` and the run failed for the wrong reason — which
        # is the same discarded-evidence shape as the control in #106.
        eval "$_body"; eval "$_ok"; eval "$_abr"; eval "$_lcs"

        _bad=0
        # Read by the `eval`'d body, which shellcheck cannot see across the boundary. These are
        # the inputs the gate under test reads; shellcheck reports the ones it cannot trace.
        # shellcheck disable=SC2034
        cbin="the-binary"
        # shellcheck disable=SC2034
        ctok="the-pkg"
        # shellcheck disable=SC2034
        LIFELOG=/dev/null
        # shellcheck disable=SC2034
        _prepath=""

        # (1) The listing shows it — delivered, and nothing else asked.
        lx() { echo "the-pkg 1.0"; return 0; }
        install_delivered be || { echo "  BAD   a package its own listing reports was judged undelivered"; _bad=1; }

        # (2) The listing does NOT show it, but the binary is on PATH — still delivered, and the
        # PATH check is what caught it. Both signals are evidence.
        path_of() { case "$1" in the-binary) echo /usr/bin/the-binary ;; *) echo "" ;; esac; }
        lx() { echo "some other package"; return 0; }
        install_delivered be || { echo "  BAD   a package on PATH was judged undelivered because its listing was empty"; _bad=1; }

        # (3) Neither — and this is the case that has been passing vacuously.
        path_of() { echo ""; }
        lx() { echo "some other package"; return 0; }
        if install_delivered be; then
            echo "  BAD   an install that delivered nothing was reported as delivered, so the removal half scores a package that was never here"
            _bad=1
        fi

        # (4) A manager whose listing cannot show the package is judged DELIVERED — it still
        # installed the thing, the listing is simply the wrong witness for it. Refusing to judge
        # would take every cabal-class backend's PATH check with it, and a backend that installed
        # correctly would be recorded as having installed nothing. This is the control for (3):
        # the same empty listing, opposite answers, and the difference is whether the lister can
        # speak for this backend at all.
        list_cannot_show() { echo "this lister reports the library DB, not executables"; }
        install_delivered cabal || {
            echo "  BAD   a backend whose listing cannot show the package was judged undelivered, so a correct cabal install would be recorded as having installed nothing"
            _bad=1
        }
        # And with a real listing plus nothing on PATH it is judged undelivered — so (4) is the
        # lister's inability and not the gate having stopped looking.
        list_cannot_show() { echo ""; }
        path_of() { echo ""; }
        if install_delivered cabal; then
            echo "  BAD   the lister exception made the gate accept a backend with nothing listed and nothing on PATH"
            _bad=1
        fi

        exit "$_bad"
    )
    if [ $? -eq 0 ]; then
        echo "  ok    $(basename "$_src") asks whether an install DELIVERED, not whether it returned 0"
    else
        BAD=$((BAD + 1))
    fi
done

# The drift register, in both harnesses. `classify_install` above degrades an ecosystem failure
# to `exhausted`, which the real-lifecycle ratchet then counts as coverage merely unmeasured —
# right for a rate-limit window, wrong for Hackage rotating its TUF root past what the image's
# cabal trusts, which no later run clears on its own. An excuse nobody can see is `|| true` with
# better manners, so the excuse needs a dated line and every run says how old it is.
#
# Both halves are arithmetic on dates, which is exactly the shape that is wrong on a leap year
# and right on every day somebody tests it by hand.
echo "== an ecosystem excuse is dated, and says how long it has stood"
for _src in $SOURCES; do
    TOTAL=$((TOTAL + 1))
    _de="$(lift days_since_epoch "$_src")"
    _dv="$(lift drift_verdict "$_src")"
    if [ -z "$_de" ] || [ -z "$_dv" ]; then
        echo "  BAD   $(basename "$_src") has no drift register: an ecosystem failure is excused forever"
        BAD=$((BAD + 1))
        continue
    fi
    (
        # Read by the lifted `drift_verdict` and by nothing shellcheck can see through
        # the `eval` below — the same suppression, and the same reason, as `TO_LONG` above.
        # shellcheck disable=SC2034
        DRIFT_WINDOW_DAYS=14
        eval "$_de"
        eval "$_dv"
        _bad=0

        # Civil-to-days, against dates computed elsewhere. The leap cases are the point: the
        # formula shifts March to the start of the year precisely so 29 February needs no
        # special case, and a wrong shift is invisible on any date in the second half of a year.
        for _pair in 1970-01-01:0 2000-02-29:11016 2024-12-31:20088 2026-02-28:20512                      2026-03-01:20513 2026-08-21:20686; do
            _want="${_pair#*:}"
            _got="$(days_since_epoch "${_pair%%:*}")"
            [ "$_got" = "$_want" ] || { echo "  BAD   ${_pair%%:*} is $_got days, not $_want"; _bad=1; }
        done
        # A date nobody can parse must not become day zero, which is 1970 and therefore an
        # excuse fifty-six years past its expiry.
        for _junk in "" "2026-8-21" "yesterday" "20260821" "2026-13-01"; do
            days_since_epoch "$_junk" >/dev/null 2>&1 &&
                { echo "  BAD   '$_junk' was accepted as a date"; _bad=1; }
        done

        _reg="$(mktemp)"
        printf 'container-linux-tools-ci 25
' > "$_reg"
        printf 'drift container-linux-tools-ci cabal 2026-08-21
' >> "$_reg"
        _today=20686   # 2026-08-21, the day the line was written

        _v="$(drift_verdict container-linux-tools-ci cabal "$_reg" "$_today")"
        [ "${_v%% *}" = ok ] || { echo "  BAD   a register line dated today does not excuse (got '$1')"; _bad=1; }
        _v="$(drift_verdict container-linux-tools-ci cabal "$_reg" $((_today + 5000)))"
        [ "${_v%% *}" = ok ] || { echo "  BAD   an old excuse stopped excusing (got '$_v')"; _bad=1; }
        [ "${_v#* }" = 5000 ] || { echo "  BAD   the run does not report how long the excuse has stood (got '$_v')"; _bad=1; }

        # **The expiry was removed on 2026-08-21, by owner ruling, and this is where it would
        # come back.** Fourteen days was right for a repository somebody reads daily and wrong
        # for one that is not: an expiry turns one upstream rotation into a board that goes red
        # and stays red, and a board that stays red is one nobody reads. The age is reported on
        # every run instead - nobody has to act, nobody can say they were not told.
        # The three ways to have no excuse, which must never be the same as having one: another
        # backend, another host class, and a register that is not there at all. The host-class
        # case is the one that matters — a drift line for the tools image must not excuse the
        # same backend on ubuntu, where the lifecycle really did stop running.
        _v="$(drift_verdict container-linux-tools-ci stack "$_reg" "$_today")"
        [ "${_v%% *}" = unrecorded ] || { echo "  BAD   an unlisted backend was excused (got '$1')"; _bad=1; }
        _v="$(drift_verdict container-linux-ubuntu-ci cabal "$_reg" "$_today")"
        [ "${_v%% *}" = unrecorded ] || { echo "  BAD   one host class excused another's backend (got '$1')"; _bad=1; }
        _v="$(drift_verdict container-linux-tools-ci cabal /no/such/register "$_today")"
        [ "${_v%% *}" = unrecorded ] || { echo "  BAD   a missing register excused something (got '$1')"; _bad=1; }
        # A line dated tomorrow is a typo, and must buy nothing.
        _v="$(drift_verdict container-linux-tools-ci cabal "$_reg" $((_today - 1)))"
        [ "${_v%% *}" = unrecorded ] || { echo "  BAD   a future-dated line was honoured (got '$1')"; _bad=1; }
        # And a `drift` line whose date is rubbish must report, not excuse.
        printf 'drift container-linux-tools-ci opam soon
' >> "$_reg"
        _v="$(drift_verdict container-linux-tools-ci opam "$_reg" "$_today")"
        [ "${_v%% *}" = unrecorded ] || { echo "  BAD   an unparseable date was honoured (got '$1')"; _bad=1; }

        rm -f "$_reg"
        exit "$_bad"
    )
    if [ $? -eq 0 ]; then
        echo "  ok    $(basename "$_src") dates an ecosystem excuse and reports its age"
    else
        BAD=$((BAD + 1))
    fi
done

# ---------------------------------------------------------------------------
# The section floor: did every section run, against `scripts/section-floor.txt`.
#
# **This is the only gate in the file whose failure is invisible in the run it judges.** Every
# other predicate here has a check that must be capable of failing; this one is the claim that
# they all did, and its own subject — a section that stops running — produces no failing check
# to find. Measured: `crash/groupkill` lost 10 of its 10 checks on one image, printed one
# `soft`, and the run ended `pass=391 fail=1 soft=5` — green, and eight checks short of the 405
# the previous `main` ran on the same image with the same backend. So both halves below are the
# whole point and both are driven: a shortfall with nothing to say for itself must FAIL, and a
# shortfall the harness ANNOUNCED must not — because a gate that punishes the instrument for
# declining to measure is a gate that gets switched off, and this one would be switched off on
# eleven images at once the first time a `SIGKILL` landed outside a transaction window.
#
# The functions are lifted from the harness, so this tests the bytes CI runs. The control is
# the first case and it matters more than the rest: a floor that cannot tell a section at its
# number from a section that was never compared reports the second as the first, and "every
# section is above its floor" is exactly what a gate holding no record at all would say.
echo "== a section below its floor is a failure, and a section that SAID why is not"
for _src in $SOURCES; do
    if ! grep -q 'print_section_tally' "$_src"; then
        # The Windows and macOS harnesses have no section tally, so there is nothing for a
        # floor to compare and nothing worth mounting one for. That is a stated gap and not a
        # second implementation of this gate — the shape that WOULD be a defect is a harness
        # that mounts the file and prints no tally, and
        # `tests/the_review_apparatus_is_rust_tests.rs` watches that one.
        TOTAL=$((TOTAL + 1))
        if grep -q 'check_section_floors' "$_src"; then
            echo "  BAD   $(basename "$_src") has a section floor and no tally: it compares every section against zero"
            BAD=$((BAD + 1))
        else
            echo "  ok    $(basename "$_src"): no section tally, so no section floor — the gap is Q12 rule 6's other half, still open"
        fi
        continue
    fi
    _body=""
    for _fn in section_floor_record section_soft_says _credit_section _record_soft \
               check_section_floors report_section_floors; do
        _b="$(lift "$_fn" "$_src")"
        if [ -z "$_b" ]; then _body=""; break; fi
        _body="$_body
$_b"
    done
    if [ -z "$_body" ]; then
        TOTAL=$((TOTAL + 1))
        echo "  BAD   $(basename "$_src") has no section floor: nothing compares what its sections ran"
        BAD=$((BAD + 1))
        continue
    fi
    (
        PASS=0; FAILC=0; SOFTC=0; FAILED_NAMES=""
        # shellcheck disable=SC2034
        SMOKE=""
        soft() { SOFTC=$((SOFTC + 1)); _record_soft "$1"; }
        eval "$_body"

        _sf_dir="$(mktemp -d)"
        SECTIONS="$_sf_dir/sections"; SECTION_CHECKS="$_sf_dir/checks"
        SECTION_SOFTS="$_sf_dir/softs"; FLOOR="$_sf_dir/section-floor.txt"
        : > "$SECTIONS"; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        # Two sections, because a gate that only ever sees one cannot be shown to be reading
        # the right one. `Bootstrap` is at its floor, `SIGKILL` is the section the finding was
        # in, and the record gives the second a `soft`-bound — the shape a real entry has.
        printf '0\tBootstrap\n1\tSIGKILL mid-transaction, then heal\n' > "$SECTIONS"
        printf '%s\n' \
            "# a comment, which the parser must skip" \
            "container-linux-ubuntu-local  Bootstrap  4" \
            "container-linux-ubuntu-local  SIGKILL mid-transaction, then heal  41  soft  22  2026-09-30  the kill opened no new entry in the write-ahead log" \
            > "$FLOOR"
        # `idx` is the section a check is credited to, which is the only thing the tally's two
        # files hold; `credit N` writes N of them against the current one.
        credit() { _i=0; while [ "$_i" -lt "$1" ]; do printf '%s\n' "$idx" >> "$SECTION_CHECKS"; _i=$((_i + 1)); done; }
        reset() { PASS=0; FAILC=0; SOFTC=0; FAILED_NAMES=""; }
        judge() {
            check_section_floors container-linux-ubuntu-local "$1" > "$_sf_dir/out" 2>&1
            report_section_floors >> "$_sf_dir/out" 2>&1
        }
        _bad=0

        # The control, and the run every other case here is a deviation of.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 41
        judge "$FLOOR"
        [ "$FAILC" -eq 0 ] || { echo "  BAD   two sections at their floors failed the run"; _bad=1; }
        grep -q "2 section(s) at or above the floor" "$_sf_dir/out" \
            || { echo "  BAD   the pass did not say how many sections it compared — which is what a gate that compared nothing would also print"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): sections at their floors pass, and the pass says it compared two"

        # The defect this exists for, in the exact shape of the measured one: the section is
        # short, nothing printed, and the run must end in a FAILURE rather than a quiet number.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 4
        judge "$FLOOR"
        [ "$FAILC" -eq 1 ] || { echo "  BAD   a section that credited 4 of 41 checks did not fail the run"; _bad=1; }
        grep -q "SIGKILL mid-transaction, then heal(4, floor 41)" "$_sf_dir/out" \
            || { echo "  BAD   the failure did not name the section, the count and the floor:"; sed -n 's/^/          /p' "$_sf_dir/out"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): a section that stopped running in silence fails, and is named"

        # The mirror, and the reason a `soft`-bound line exists at all: the same shortfall, with
        # the harness announcing it. Matched with `grep -F` against what this run actually
        # printed, so an excuse is worth exactly as much as the sentence in it — and it is
        # reported, because an excuse honoured silently is how a floor becomes a comment.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 22
        _record_soft "crash/groupkill: the kill opened no new entry in the write-ahead log (0 before, 0 after), so this iteration measured no recovery"
        judge "$FLOOR"
        [ "$FAILC" -eq 0 ] || { echo "  BAD   a shortfall the harness announced was scored as a failure"; _bad=1; }
        grep -q "excused by a recorded" "$_sf_dir/out" \
            || { echo "  BAD   the excuse was honoured silently"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): a shortfall with the soft that explains it is excused, loudly"

        # **And the count still binds.** The same soft, the same section, and a run that lost
        # more than the excuse covers: the excusal says 22, this measured 4, and the reason is
        # not a reason for the other eighteen checks. A bound that did not bind would excuse
        # the measured collapse as thoroughly as the one it was written for.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 4
        _record_soft "crash/groupkill: the kill opened no new entry in the write-ahead log (0 before, 0 after), so this iteration measured no recovery"
        judge "$FLOOR"
        [ "$FAILC" -eq 1 ] || { echo "  BAD   an excusal covered a count below the one it was written for"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): the excuse's count binds — 4 is not the 22 it was written for"

        # **And the text binds.** A record that excuses a shortfall only while the run prints
        # THAT soft cannot be used to absorb a different one, which is the whole reason the
        # line carries words and not just a number.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 22
        _record_soft "crash/groupkill: something else went wrong entirely"
        judge "$FLOOR"
        [ "$FAILC" -eq 1 ] || { echo "  BAD   a soft this run did not print was accepted as the excuse"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): an excuse is honoured only by the soft it names"

        # A `soft` in a DIFFERENT section does not excuse this one. Attribution is the whole
        # mechanism: an excuse matching any soft anywhere in the run would be honoured by the
        # run's least informative line.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; credit 22
        _record_soft "crash/groupkill: the kill opened no new entry in the write-ahead log (0 before, 0 after), so this iteration measured no recovery"
        judge "$FLOOR"
        [ "$FAILC" -eq 1 ] || { echo "  BAD   a soft attributed to another section excused this one"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): a soft excuses only the section that printed it"

        # A run that beat its record, which is the other half of a ratchet: it passes AND it
        # prints the edit, because a floor that can only fall is a ceiling nobody set.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 9; idx=1; credit 41
        judge "$FLOOR"
        [ "$FAILC" -eq 0 ] || { echo "  BAD   a section above its floor failed"; _bad=1; }
        grep -q 'ratchet up:.*container-linux-ubuntu-local  Bootstrap  9' "$_sf_dir/out" \
            || { echo "  BAD   a section above its floor did not print the line to add:"; sed -n 's/^/          /p' "$_sf_dir/out"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): beating the record passes and prints the edit"

        # **The gate that is not in force must say so, loudly.** A file that is not mounted —
        # the state N-5 describes for the ratchet that shares this shape, on five legs at once,
        # with every one of them green — lands in the branch that compared nothing, and
        # "nothing is short" is what that branch must never report.
        reset
        judge "$_sf_dir/no-such-file"
        [ "$FAILC" -eq 1 ] || { echo "  BAD   a missing floor file passed the gate"; _bad=1; }
        grep -q "no record for container-linux-ubuntu-local" "$_sf_dir/out" \
            || { echo "  BAD   the missing file was not named as a missing record"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): a class with no record at all is a failure, not a pass"

        # A class with no line for ONE section is neither passed nor failed, and says what to
        # add. Q12's rule 4: a gate that fails the first time it meets a new platform is a gate
        # that stops people adding platforms — and it is still not a pass, because a record
        # that is not there compares nothing.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 4; idx=1; credit 41
        printf '2\tA section added after this file was written\n' >> "$SECTIONS"
        judge "$FLOOR"
        [ "$FAILC" -eq 0 ] || { echo "  BAD   a section with no record failed the run"; _bad=1; }
        grep -q "no record yet for: A section added after this file was written (0)" "$_sf_dir/out" \
            || { echo "  BAD   the unrecorded section was not named, so nothing says to add it:"; sed -n 's/^/          /p' "$_sf_dir/out"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): an unrecorded section is reported and counted as neither"

        # SMOKE_ONLY measures a different run, and a floor over it reports the mode. This is
        # the branch that keeps a documented local invocation (`SMOKE_ONLY=1`) from reddening
        # on every image, and it has to be a `soft` — a PASS would be a section compared
        # against nothing and scoring perfectly.
        reset; : > "$SECTION_CHECKS"; : > "$SECTION_SOFTS"
        idx=0; credit 0; idx=1; credit 0
        # shellcheck disable=SC2034
        SMOKE=1
        judge "$FLOOR"
        [ "$SOFTC" -eq 1 ] || { echo "  BAD   SMOKE_ONLY was neither softened nor refused"; _bad=1; }
        [ "$PASS" -eq 0 ] || { echo "  BAD   SMOKE_ONLY scored a pass on the section floor"; _bad=1; }
        [ "$_bad" = 0 ] && echo "  ok    $(basename "$_src"): SMOKE_ONLY is not judged, and does not pass either"

        rm -rf "$_sf_dir"
        exit "$_bad"
    )
    if [ $? -eq 0 ]; then :; else BAD=$((BAD + 1)); fi
    TOTAL=$((TOTAL + 1))
done

# The coverage audit's floor. Both harnesses take `ALL_BACKENDS` and `HELP_CMDS` from the
# program under test and then assert set-containment — and a `for` over an empty list runs
# zero times, leaves the "untouched" string empty, and PASSes. Measured under a do-nothing
# stub: the audit printed "0 in --help … 0 registered" and passed both meta-checks. An audit
# that enumerates nothing scoring perfect coverage is the exact shape of every finding in this
# file, applied to the thing that was supposed to find them.
echo "== a collapsed registry cannot pass the coverage audit"
for _src in $SOURCES; do
    TOTAL=$((TOTAL + 1))
    _body="$(lift too_few_to_audit "$_src")"
    if [ -z "$_body" ]; then
        echo "  BAD   $(basename "$_src") has no too_few_to_audit(): its coverage audit has no floor"
        BAD=$((BAD + 1))
        continue
    fi
    eval "$_body"
    # 0 and 1 are collapse; a real registry is 48 on Windows and 56 on Ubuntu, and a real
    # `--help` is ~55 subcommands. Both directions, so a floor of zero cannot pass this.
    if too_few_to_audit 10 0 && too_few_to_audit 10 1 && ! too_few_to_audit 10 48; then
        echo "  ok    $(basename "$_src") refuses to audit a registry that came back empty"
    else
        echo "  BAD   $(basename "$_src") too_few_to_audit() does not tell collapse from a real registry"
        BAD=$((BAD + 1))
    fi
done

# The mutation gate's own collapse case. `grep -c` prints `0` AND exits 1 when it matches
# nothing, so `COUNT=$(grep -c … || echo 0)` captured the two-line string "0\n0". Both of the
# gate's guards then died with "integer expected", `[` returning an error took the else branch
# of each `if`, and the script fell through to its success message — reporting ok, exiting 0,
# in exactly the total-collapse case the guards exist to catch. A gate that cannot fail is the
# thing this whole file is about.
echo "== the mutation gate fails when the harness produced nothing at all"
TOTAL=$((TOTAL + 1))
_mg="$(dirname "$0")/harness-mutation-test.sh"
_silent="$(mktemp -d)/silent-harness.sh"
printf '#!/usr/bin/env bash\nexit 0\n' > "$_silent"
chmod +x "$_silent"
if bash "$_mg" "$_silent" --check >/tmp/mg.out 2>&1; then
    echo "  BAD   the mutation gate passed a harness that emitted no checks at all:"
    sed -n 's/^/        /p' /tmp/mg.out | tail -5
    BAD=$((BAD + 1))
else
    echo "  ok    a harness that emits no checks fails the mutation gate"
fi
rm -rf "$(dirname "$_silent")"

# And the other side of the same gate (R-7). A ceiling on survivors cannot tell "the checks got
# stronger" from "the checks were deleted": pointed at a harness with three checks, the gate
# reported `ok: 2 survivors, within the budget of 92; 1 checks did their job` and exited 0. The
# collapse case above only fires when a harness emits NOTHING; a harness reduced from 121 checks
# to 3 emits plenty.
echo "== the mutation gate fails when a harness still runs but its assertions are gone"
TOTAL=$((TOTAL + 1))
_tinydir="$(mktemp -d)"
_tiny="$_tinydir/tiny-harness.sh"
cat > "$_tiny" <<'TINYEOF'
#!/usr/bin/env bash
LX="${SHALL:-shall}"
ok()  { echo "  PASS  $1"; }
nok() { echo "  FAIL  $1"; }
if "$LX" --version >/dev/null 2>&1; then ok "Shall runs"; else nok "Shall runs"; fi
if "$LX" init      >/dev/null 2>&1; then ok "init runs";  else nok "init runs";  fi
if "$LX" eval | grep -q schema;       then ok "eval emits a model"; else nok "eval emits a model"; fi
TINYEOF
chmod +x "$_tiny"
if bash "$_mg" "$_tiny" --check >/tmp/mg-floor.out 2>&1; then
    echo "  BAD   the mutation gate passed a harness with three checks and one real assertion:"
    sed -n 's/^/        /p' /tmp/mg-floor.out | tail -3
    BAD=$((BAD + 1))
else
    echo "  ok    a harness whose assertions were deleted fails the mutation gate"
fi
rm -rf "$_tinydir"

# The six predicates that used to live here read `ci.yml`, the release scripts, the Dockerfiles
# and the harnesses as text and never ran a script or entered a container: gate parity, orphan
# scripts, function-defined-before-called, CRLF endings, floor mounts, image identity. They are
# `tests/the_review_apparatus_is_rust_tests.rs` now, where they fail in `cargo test` next to the
# twenty-seven other gates that read this repo, rather than at the end of a release script.
#
# What is left in this file is the half that cannot move: `lift` pulls function bodies out of
# the harnesses and runs them, which tests the bytes CI actually executes, in the interpreter
# that executes them.
# The register's own arithmetic. Two files tracked one number by hand and disagreed four ways
# at once — `decisions.md` said 109, 107 and 104 in three places, `SPEC.md` said 107. Counting
# is the fix; checking the count on every push is what stops it coming back.
echo "== the decision register's counts match the register"
TOTAL=$((TOTAL + 1))
if sh "$(dirname "$0")/decision-count.sh" --check >/tmp/dc.out 2>&1; then
    echo "  ok    every documented decision count matches the register"
else
    echo "  BAD   documented decision counts disagree with the register:"
    sed -n 's/^  BAD   /        /p' /tmp/dc.out
    BAD=$((BAD + 1))
fi

echo "--------------------------------------------------------------"
echo " harness predicates: $((TOTAL - BAD))/$TOTAL ok"
[ "$BAD" = 0 ] || { echo " FAILED"; exit 1; }

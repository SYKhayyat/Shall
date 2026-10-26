#!/bin/sh
# Bring the guix daemon up, install the harness's fixtures, then hand over to the harness.
#
# The daemon start lives in `guix-daemon-up.sh`, because the terminator probe step replaces this
# entrypoint with `sh` and needs the daemon just as much — see that file's header. What stays
# here is what only the lifecycle harness needs.
set -u

sh /usr/local/bin/guix-daemon-up || exit 1

# **The one fixture that has to come from guix.** A guix profile starts empty, and the removal
# guard needs a victim: a package from Shall's OWN protected list that this machine has
# INSTALLED, so the guard's refusal can be witnessed. The harness looks for it among the
# packages `shall list --backend guix` reports, so the victim has to be a guix package —
# `bash`, which is on Shall's protected list by default.
#
# `git` used to be here too, and this was the wrong place for it: guix's `git` failed to build
# on this image at `glib-2.86.5` — measured, 2026-10-26, and **with a C toolchain present**, so
# the compiler was not the missing piece. The history and rollback proofs want a `git` BINARY on
# PATH, and every other image in this matrix uses its base image's git; Shall's git-backed
# history is not a guix feature. So the image carries git now, and this carries what only guix
# can carry.
#
# **Failure is reported and not fatal.** This is a fixture for another people's checks; a guix
# that cannot fetch it is worth knowing about, and it is not a reason to run none of the sweep.
# The log is kept and its last forty lines are printed, because the failure used to say only
# "cannot build derivation ...: 1 dependencies couldn't be built" — a dependency and not a cause.
if guix install bash >/tmp/guix-fixture-bash.log 2>&1; then
    echo "guix fixture: bash installed"
else
    echo "WARNING: could not install the bash fixture; the removal-guard proof reports" >&2
    echo "         itself unrunnable. /tmp/guix-fixture-bash.log says:" >&2
    tail -40 /tmp/guix-fixture-bash.log >&2
fi
hash -r 2>/dev/null || true

exec sh /src/docker/integration/run-in-container.sh "$@"

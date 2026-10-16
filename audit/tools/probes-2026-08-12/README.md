# The 12 August probe drivers

Fifty-five shell scripts that drove Shall through real containers on 2026-08-12, recovered on
2026-10-16 from the home directory of a WSL Ubuntu distro that was about to be unregistered. They sat
loose in `$HOME`; nothing else referenced them, and they were run by hand.

They are kept because the work they did is the work the integration harness does, one experiment at a
time, and because a probe whose output lives in `audit/results/` should have its tool beside it. The
outputs were promoted separately — `audit/results/` holds the runs, not this directory.

## The pattern every one of them follows

    WORK=$HOME/ctr37
    rm -rf $WORK && mkdir -p $WORK
    cp $HOME/shallbench/target/release/shall $WORK/      # the binary under test
    cat > $WORK/inner.sh <<'INNER' … INNER               # the checks, written fresh each run
    chmod +x $WORK/inner.sh
    docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh

Inside, the workdir appears as `/opt/b`, so the checks run `/opt/b/shall --config-dir /cfg
--data-dir /data`, declare packages, sync, and print what they found.

Each driver writes its own `inner.sh`, which is why the 37 leftover `inner.sh` files found beside them
in `$HOME/ctr*/` were run outputs rather than inputs: none of these reads one. (`rpm.sh` is the only
script that mounts a `ctr*/inner.sh`, and it writes that file itself first.) Those leftovers were left
behind when the distro went; nothing here needs them.

## To run them again

    build       $HOME/shallbench/target/release/shall
    config      $HOME/shallcfg  and  $HOME/shalldata     (the host-side drivers use these)
    staging     $HOME/bmnt — 8 drivers mount it at /opt/b:
                al.sh hc.sh hold.sh hr.sh hw.sh nb.sh nb2.sh sha.sh
    images      ubuntu:24.04 (40 drivers), fedora:41 (rpm.sh), shall-tools (hold.sh, nb.sh,
                nb2.sh, al.sh) — the last is a locally built toolbox image, not a registry pull
    workspace   $HOME/ctrNN, created and removed by each driver
    host tools  hyperfine and strace (bench.sh hf.sh stats.sh statx.sh strace.sh trace.sh where.sh)

Two things to repair first:

    lwmsync.sh    SRC=/mnt/c/Users/Administrator/Videos/LatticeWM — that checkout was deleted on
                  2026-10-16, so repoint SRC at wherever the tree lives now.
    shall-tools   rebuild it from the repo's Dockerfile, or the bmnt drivers cannot start.

## Committed verbatim, and the one known nit

These are the scripts exactly as they were found, absolute `$HOME/shallbench` paths and all, so that a
diff against the recovered originals is empty. `shellcheck -S warning` reports one finding across the
set — `bench.sh:21` splits command output where `mapfile` is preferred (`SC2207`). It is not a CI red:
that check globs `scripts/*.sh`, `docker/**/*.sh` and `.githooks/*`, and this directory is under none
of them.

**They are executable in the index, and that took a second attempt.** A script that comes off a Windows
filesystem lands as `100644`, because Windows has no executable bit to carry; the distro had these as
`-rwxr-xr-x`. `every_shipped_script_is_executable_tests` asks git about the *index* rather than the
working tree — the index is the only copy a clone receives — and it failed the first commit of these
files, naming all fifty-five, with the fix in its message: `git update-index --chmod=+x`. So the tree
holds 55 × `100755` and this file at `100644`, which the gate's second half requires of anything without
a shebang. Worth remembering if scripts from a Windows box are ever added here again.

## Which is which

Host-side, no container:

    bench.sh    Interleaved A/B benchmark so background load drift cancels out — warms both once, so
                it measures steady state rather than first touch.
    diag.sh     Prints the live config: priority file contents and service state.
    hf.sh       hyperfine timing runs against the configured binary.
    where.sh    strace -e execve per command, counting the subprocesses each one spawns.
    stats.sh statx.sh strace.sh trace.sh    syscall-level profiles of a sync.
    gt.sh       A throwaway lifecycle under /tmp/g.
    lwmsync.sh  Mirror the Windows tree into WSL with LF endings, then run a make target.

Named container experiments — the rest are unnamed variants from the same session, and their checks
are readable at the top of each file:

    al.sh    "Drive a real lifecycle for many backends inside the tools image, and ask the B0
             question for each: after Shall removes it, does `shall list` still claim it is
             installed?"
    b0i.sh   Mass-removal guard: `shall info` for a package that was removed, and for one that was
             never installed at all.
    d2.sh    "now the other direction: delete the declaration, sync must REMOVE it".
    fp.sh    "exit codes the program documents: 0 converged, 1 failed, 2 differences, 3 refused".
    hold.sh  "switch declaration from a version pin to a hold".
    mu.sh    "ground truth per backend, asked of the MANAGER, never of Shall — what SHALL believes".
    nb.sh    "a web: download with a deliberately wrong sha256 must FAIL, not install".
    nb2.sh   "both npm and gem have a package called 'json'. priority lists npm first. hold is-odd,
             then upgrade — is-odd must stay" held.
    pl.sh    "run a manifest through the PLANNER and report rc + one line of what it said".
    r3.sh    "prepare scripts for hyperfine (-N gives no shell, so these must be files)".
    r3b.sh   Prepare: declare the three, sync so Shall manages them, then undeclare so the next sync
             removes them — the benchmark's setup half.
    vb.sh    "metapac: declare something already installed -> sync is a no-op / shall: same
             declaration -> sync is a no-op".

Thirteen carry a comment saying what they were for; the other forty-two follow the identical
stage-and-run pattern without one.

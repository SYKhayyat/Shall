//! `sbom` and `export` ask every manager one at a time.
//!
//! `latency.rs`'s `Shape` is a collapse detector for exactly this: overlap ≈ 1.0 with one wave
//! per child is the signature of a serial fan-out, and the floors were calibrated across three
//! platforms so they hold on a host nobody has measured. `the_fan_out_commands_still_fan_out`
//! drives it — and drives `list`, alone, under a name in the plural.
//!
//! Measured on Windows, 133 declarations, 17 live managers, debug build, quiet box, over the
//! same set of 21 child commands:
//!
//! ```text
//! shall list           3.59s wall   21 children summing to 20.99s   5.9x   1 wave
//! shall check health   3.47s wall   23 children summing to 22.03s   6.3x   2 waves
//! shall plan           3.68s wall   24 children summing to 14.03s   3.8x   3 waves
//! shall sbom          11.12s wall   21 children summing to 11.04s   1.0x  21 waves
//! shall export         9.42s wall   21 children summing to  9.36s   1.0x  21 waves
//! ```
//!
//! `list` overlaps the same twenty-one commands 5.9×. `sbom` and `export` overlap them not at
//! all — 1.0×, one wave each, the exact signature `Shape` names. Both fail *both* of its
//! bounds: 1.0 is under the 1.5 floor, and 21 waves is over a ceiling of 10.
//!
//! **The machinery is not missing, it is unused.** Nothing here asks for a new mechanism; the
//! concurrency that gives `list` its 5.9× is in the tree and these two commands do not reach
//! for it. On this host that is nine seconds a user waits for a report that a manager could
//! have answered in under two.
//!
//! **Why nothing caught it.** `Class::of` files both under `ConfigOnly` — *"Reads files,
//! answers, stops"* — and `Class::shape()` returns `None` for that class, so the detector never
//! looks at the two commands that trip it. The wall-clock budget the misfiling *does* give them
//! is 5s, which neither can meet: `shall sbom` prints its own `WARN … budgeted 5s` on an
//! ordinary run. A warning that fires every time is one users learn to scroll past, and it is
//! the same warning that exists to catch the next 98-second `info`.
//!
//! So this is one defect with two faces, and the second is the more expensive: the class table
//! is wrong, which simultaneously suppresses the check that would fail and enables one that
//! cries wolf.
//!
//! **The bounds are `Shape`'s, asked of the type.** Borrowed from `list` deliberately: these
//! commands carry no shape of their own, and that is the thing under test.
//!
//! **The overlap ratio is no longer judged here, and the CI run that failed is why.** On the
//! Windows runner `sbom` and `export` report `1.2x overlap over 7 children` against a 1.25x
//! floor — and the table beside it shows two children in flight together, which is not a serial
//! run. The difference is the host: starting a child process there costs about as long as the
//! child then runs (7 children summing to 5.23s inside 4.25s of wall clock), so every span begins
//! late and `sum / wall` reads the process-startup cost rather than the scheduling. `list` passes
//! the same floor on the same host only because it declares seventeen managers to `sbom`'s five,
//! and a ratio is bounded by the fan-out's width — with five chains and a `max_parallel` of the
//! core count, 1.2x is what a correct fan-out scores there.
//!
//! So the assertion is the one that cannot be corrupted by a slow host: the run was **more than
//! one wide at some instant**. A `for` loop that awaits each child is exactly 1 wide whatever a
//! child costs, and that — not a number near a floor — is the defect #107 is about.
//!
//! **`waves` goes with the ratio, and for a second reason the Linux runner supplied.** With
//! `max_parallel` at the core count — two on that runner — five managers with a `list` and a
//! `prefix` each produce six waves however well they are scheduled: a wave is counted wherever the
//! run goes quiet, and a two-slot budget over five chains goes quiet by arithmetic. `Shape`'s
//! ceiling is `children / waves_per_child`, calibrated on one child per manager, so it is reading
//! a budget and a chain length rather than a scheduler. The ceiling still applies where it was
//! measured and is meaningful — `list`, one child per manager, in `latency_budget_tests`.
//!
//! The precise, host-free version of this question lives in
//! `a_fan_out_of_managers_is_asked_concurrently_tests`, over mock managers and no clock reading.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A config that declares what this host actually has.
///
/// **`adopt`, and not an empty `init`.** `list` fans out over the machine whatever the manifest
/// says, so the existing shape gate works from a bare directory. `sbom` and `export` report on
/// what is *declared*, so a bare directory makes them ask nothing at all and the gate skips
/// itself into a pass — which is the failure mode this file is about, arrived at from the other
/// side. Adoption gives them the manifest a user has on day one.
fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shall-fanout-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for args in [&["init"][..], &["adopt", "-y"][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_shall"))
            .args(args)
            .env("SHALL_CONFIG_DIR", dir.join("config"))
            .env("SHALL_DATA_DIR", dir.join("data"))
            .stdin(std::process::Stdio::null())
            .output()
            .expect("the binary should run");
        assert!(out.status.success(), "`shall {}` failed", args.join(" "));
    }
    dir
}

/// The `Timings:` summary line, off stderr where `--timings` puts it.
///
/// Run inside the fixture: `export` writes its output file into the working directory, so a
/// child inheriting the test runner's cwd leaves `package.json` in the repository — five of
/// them, once the collision suffixes start.
/// The summary line to parse, and the whole report to quote when the assertion fails.
///
/// **Both, and the whole report is the half that matters.** `timing::report` already prints a
/// per-child table under the summary line — `at`, `took`, `command`, one row per label — and this
/// helper used to throw it away, keeping only the aggregate. That is why `#107` could name neither
/// the seven commands nor the shape of their serialisation: the evidence was in the captured
/// string and not in the panic. A failing CI run of this gate now prints the table, so the next
/// person to read it sees which managers were asked, when each started, and how long each took —
/// which is the whole of the question this file asks.
fn timings(dir: &Path, subcommand: &str) -> (String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_shall"))
        .args(["--timings", subcommand])
        .env("SHALL_CONFIG_DIR", dir.join("config"))
        .env("SHALL_DATA_DIR", dir.join("data"))
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary should run");
    let report = String::from_utf8_lossy(&out.stderr).into_owned();
    let line = report
        .lines()
        .find(|l| l.starts_with("Timings:"))
        .unwrap_or_else(|| {
            panic!(
                "`shall --timings {subcommand}` printed no `Timings:` line; the instrument this \
                 gate reads is gone:\n{report}"
            )
        })
        .to_string();
    (line, report)
}

fn number_before(line: &str, unit: &str) -> Option<f64> {
    let at = line.find(unit)?;
    line[..at]
        .rsplit(|c: char| !(c.is_ascii_digit() || c == '.'))
        .find(|t| !t.is_empty())?
        .parse()
        .ok()
}

/// One command, read for whether it ever had two children in flight at once — or a loud skip when
/// the host cannot answer.
fn assert_not_serial(subcommand: &str) {
    let dir = fresh(subcommand);
    let (line, report) = timings(&dir, subcommand);

    let Some(children) = number_before(&line, " child command(s)") else {
        eprintln!("fan-out shape ({subcommand}): SKIPPED — no child commands here:\n{report}");
        return;
    };
    let children = children as usize;
    let shape = shall::core::latency::Class::of("list")
        .shape()
        .expect("`list` asks every manager, so it carries a shape budget");
    if children < shape.min_children {
        eprintln!(
            "fan-out shape ({subcommand}): SKIPPED — {children} child command(s), too few for \
             a width to say anything about a scheduler (`list`'s own floor is {}):\n{report}",
            shape.min_children
        );
        return;
    }

    let width = number_before(&line, " wide").unwrap_or_else(|| {
        panic!(
            "`shall --timings {subcommand}` printed no width, so there is no reading of this run \
             a slow host cannot corrupt; the instrument is gone:\n{report}"
        )
    }) as usize;

    // **Child commands, not managers.** A five-manager host spawns seven child commands here —
    // `npm list` and `npm prefix`, `pipx list` and `pipx environment` — so a message that called
    // seven "managers" would be naming a number that is neither a manager count nor a command
    // count. It is the command count below, which is what the table underneath lists.
    //
    // **Width, not the overlap ratio.** See this file's header: on the Windows runner the ratio
    // read 1.2x for a run that had two children in flight, because the host spends about as long
    // creating a process as the process then runs. A serial `for` loop has width 1 on every host,
    // which is the whole of what this gate can honestly claim from a wall clock.
    assert!(
        width >= 2,
        "`shall {subcommand}` spawned {children} child command(s) and was never more than {width} \
         wide — no two children were in flight at the same instant, which is what a `for` loop \
         awaiting each child looks like, and `shall list` asks these same managers several at a \
         time on any host. The table below names every child, when it started, and how long it \
         took.\n{report}"
    );
}

/// The control, and it must stay green: the same assertions over the command that already
/// fans out. If this ever goes red the host is the problem, not the two below it.
#[test]
fn list_still_fans_out_which_is_what_makes_the_rest_a_finding() {
    assert_not_serial("list");
}

#[test]
fn sbom_asks_the_managers_concurrently() {
    assert_not_serial("sbom");
}

#[test]
fn export_asks_the_managers_concurrently() {
    assert_not_serial("export");
}

/// The class table is wrong, and it says so in its own numbers.
///
/// `ConfigOnly` is documented as *"Reads files, answers, stops"* and every other command in it
/// is flat at ~150 ms whatever the manifest holds. These two spawn one child process per
/// manager. The label is what turns off the shape gate above and turns on a 5-second budget
/// they cannot meet, so it is not a naming quibble — it is the reason both halves misfire.
#[test]
fn a_command_that_spawns_a_child_per_manager_is_not_config_only() {
    use shall::core::latency::Class;

    let mut misfiled = Vec::new();
    for subcommand in ["sbom", "export"] {
        let dir = fresh(&format!("class-{subcommand}"));
        let (line, report) = timings(&dir, subcommand);
        let Some(children) = number_before(&line, " child command(s)") else {
            continue; // No managers on this host; the class cannot be judged from here.
        };
        if children as usize >= Class::of("list").shape().unwrap().min_children
            && Class::of(subcommand).shape().is_none()
        {
            misfiled.push(format!(
                "`{subcommand}` is {:?} and spawned {} child command(s):\n{report}",
                Class::of(subcommand),
                children as usize
            ));
        }
    }

    assert!(
        misfiled.is_empty(),
        "{} command(s) ask every manager and are classed as reading only files, which is what \
         exempts them from the shape budget:\n  {}",
        misfiled.len(),
        misfiled.join("\n  ")
    );
}

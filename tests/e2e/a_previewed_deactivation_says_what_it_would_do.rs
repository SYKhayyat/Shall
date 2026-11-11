//! `shall --dry-run deactivate X` answers with silence, and silence is not a preview.
//!
//! `activate` prints nothing by design — it only writes `active` — and the harness had already been
//! told that, so its `deactivate` check waited for a line and got none, on every image, for as long
//! as the check has existed (#153). Reproduced by hand:
//!
//! ```text
//! $ shall profile create HarnessProfile
//! Created profile 'HarnessProfile' at the profiles directory.
//! $ shall --dry-run deactivate HarnessProfile ; echo "rc=$?"
//! rc=0
//! ```
//!
//! Two defects, and the second is why the first was invisible. `deactivate` *had* a preview arm,
//! and it was a `tracing::info!` — and the default level is `warn` (`entrypoint::log_level_from_argv`),
//! so 46 preview sites in `src/` print nothing unless `-v` was passed. `core::dry_run`'s own doc
//! comment said `info!` was \"the default level, so the line is seen\"; it is not, and the comment was
//! the reason nobody checked.
//!
//! So the assertions here are two: the preview speaks at the **default** verbosity (no `-v`), and it
//! changes nothing. The second is paired with a real deactivation, because \"the file still says the
//! profile\" is also true of a command that never recognised the name.

use std::path::PathBuf;
use std::process::Command;

struct Sandbox {
    root: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Self {
        let root = tempfile::TempDir::new().unwrap();
        let sandbox = Self { root };
        let (out, code) = sandbox.run(&["init"]);
        assert_eq!(code, 0, "the fixture's `shall init` failed:\n{out}");
        for profile in ["Work", "HarnessProfile"] {
            let (out, code) = sandbox.run(&["profile", "create", profile]);
            assert_eq!(code, 0, "the fixture could not create `{profile}`:\n{out}");
        }
        let (out, code) = sandbox.run(&["-y", "activate", "Work", "HarnessProfile"]);
        assert_eq!(
            code, 0,
            "the fixture could not activate the two profiles:\n{out}"
        );
        sandbox
    }

    fn cfg(&self) -> PathBuf {
        self.root.path().join("config")
    }

    fn active(&self) -> String {
        std::fs::read_to_string(self.cfg().join("active")).unwrap_or_default()
    }

    fn run(&self, args: &[&str]) -> (String, i32) {
        let out = Command::new(env!("CARGO_BIN_EXE_shall"))
            .args(args)
            .env("SHALL_CONFIG_DIR", self.cfg())
            .env("SHALL_DATA_DIR", self.root.path().join("data"))
            .stdin(std::process::Stdio::null())
            .output()
            .expect("the binary should run");
        (
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
            out.status.code().unwrap_or(-1),
        )
    }
}

/// **The preview says what would leave, at the verbosity the user typed.** No `-v` anywhere in this
/// test: a preview that needs a flag to be seen is the defect, not the workaround.
#[test]
fn a_preview_names_the_profile_it_would_drop() {
    let sandbox = Sandbox::new();

    let (out, code) = sandbox.run(&["--dry-run", "deactivate", "HarnessProfile"]);
    assert_eq!(code, 0, "a preview is not a failure:\n{out}");
    assert!(
        out.contains("[DRY-RUN]"),
        "the preview printed nothing at the default verbosity, which is the command this test \
         exists for — a preview that says nothing cannot be told from a run that did nothing:\n{out}"
    );
    assert!(
        out.contains("HarnessProfile"),
        "the preview has to name what it would drop, or the user cannot tell which of several \
         names it is about:\n{out}"
    );
    assert!(
        sandbox.active().contains("HarnessProfile"),
        "the preview changed `active` — a preview that performs the change is the other half of \
         the same bug:\n{}",
        sandbox.active()
    );

    // The control, and it is the same name through the same path: without it, "the preview named
    // it" could be a preview of a name `deactivate` does not understand either.
    let (out, code) = sandbox.run(&["-y", "deactivate", "HarnessProfile"]);
    assert_eq!(code, 0, "the real deactivation failed:\n{out}");
    assert!(
        !sandbox.active().contains("HarnessProfile"),
        "the real run left the profile in `active`, so the preview above was never about it:\n{}",
        sandbox.active()
    );
}

/// **Nothing to do is still an answer.** This is the harness's case, and the one that printed not
/// one byte: the profile exists and is not active, so the honest preview is "nothing would change,
/// and here is why".
#[test]
fn a_preview_of_a_profile_that_is_not_active_says_so() {
    let sandbox = Sandbox::new();
    let (out, code) = sandbox.run(&["--dry-run", "deactivate", "NotActiveAnywhere"]);
    // A name no profile file declares is still a name the user asked about; the run answers.
    assert_eq!(
        code, 0,
        "a preview of an unknown name is not a failure:\n{out}"
    );
    assert!(
        out.contains("NotActiveAnywhere") || out.contains("not active"),
        "the preview of a profile that is not active said nothing at all:\n{out}"
    );

    // And the control above, in the same shape: a name that *is* active takes the other branch.
    let (out, _) = sandbox.run(&["--dry-run", "deactivate", "Work"]);
    assert!(
        out.contains("would deactivate Work"),
        "a preview of an active profile did not say it would deactivate it:\n{out}"
    );
}

/// **The other 45 sites are the family.** `would!` is one macro and one filter directive now, so this
/// asserts the mechanism rather than a second verb: a preview at the default level prints its
/// marker. `lock` is chosen because it writes a file and has no other output to confuse the check.
#[test]
fn a_preview_elsewhere_in_the_program_is_also_seen_by_default() {
    let sandbox = Sandbox::new();
    let (out, code) = sandbox.run(&["--dry-run", "lock"]);
    assert_eq!(code, 0, "a previewed lock is not a failure:\n{out}");
    assert!(
        out.contains("[DRY-RUN]"),
        "another verb's preview is still silent at the default verbosity, so #153's cause is \
         live and the deactivate fix only moved the symptom:\n{out}"
    );
}

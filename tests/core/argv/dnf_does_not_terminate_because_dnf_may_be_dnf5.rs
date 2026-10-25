use super::*;

/// Measured in the `tools` container, not inferred from a family resemblance.
///
/// Both were listed as terminating and neither had been asked. asdf answered `No such
/// plugin: --`; spack answered `No package matches the query: -- <name>` and, on a spec,
/// `string index out of range`. The grader saw the second one arrive as `Spec ~~zlib has
/// no name` after a real install, which is the same parser and a less legible message.
///
/// The header of this file already states the rule this broke: the default is "does not
/// terminate", and a manager joins the terminating set **when someone has checked its
/// argument parser**. Two joined without that, and a `--` a manager reads as a package
/// name turns every install through it into a failure.
/// **A name is not an implementation.** Fedora 41 replaced the Python `dnf` with the C++
/// `dnf5` and kept the name, so `/usr/bin/dnf` parses arguments with something that has
/// never seen getopt — and the row that said otherwise said it on the getopt *inference*,
/// which is the move the header of this file warns about and which has now broken five
/// managers rather than four.
///
/// What it cost: every `dnf` install on Fedora, from `search` outward. The container leg
/// that would have caught it in July was in a workflow file that had not parsed since
/// 2026-08-09, so the first run that could see it is the first run there was.
#[test]
fn dnf_does_not_terminate_because_dnf_may_be_dnf5() {
    assert!(!terminates_options("dnf"));
    assert!(!terminates_options("dnf5"));
    // The siblings, because they are the same program under other names: `yum` is a
    // symlink to dnf and dnf5 provides `microdnf`. Fixing one spelling and leaving these
    // would have left the identical defect live on the identical machines.
    assert!(!terminates_options("yum"));
    assert!(!terminates_options("microdnf"));
    // `rpm` is not dnf5 under any name, so it keeps its terminator.
    assert!(terminates_options("rpm"));
}

//! `shall try` — rehearse this config on a clean machine (XIII.11, U12).
//!
//! The question `plan` cannot answer: *would this config work on a machine that is not mine?*
//! A plan is computed against the host's installed set, its backends and its quirks, so a
//! config that only works because of something already on this laptop looks fine right up
//! until it reaches the second machine.
//!
//! **Reusing the Phase 6 images (U12, ruled 2026-07-24).** debian/alpine/arch are already built
//! and cover most hosts; a config-named base is the second step, not the blocker. The value is
//! the rehearsal existing at all.
//!
//! **The host is not touched.** The config goes in read-only and Shall's data directory inside
//! the container is a throwaway, so the rehearsal cannot install anything here, cannot write to
//! the repo, and cannot leave a registry behind.
//!
//! Pure: choosing a runtime, building the argv, and the refusals. Running it is the caller's.

/// A container runtime, in the order `try` prefers them.
///
/// Docker first because it is what most machines have; podman second because it is what the
/// machines that deliberately do not have Docker have. Both take the same arguments for
/// everything `try` needs, which is why this is a name rather than a trait.
pub const RUNTIMES: [&str; 2] = ["docker", "podman"];

/// The images Phase 6 already builds, and what each is for.
pub const IMAGES: [(&str, &str); 3] = [
    ("shall-it-ubuntu", "debian/ubuntu, apt"),
    ("shall-it-alpine", "alpine, apk"),
    ("shall-it-arch", "arch, pacman"),
];

/// The image `try` uses when none is named.
pub const DEFAULT_IMAGE: &str = "shall-it-ubuntu";

/// The first runtime that is actually here.
pub fn pick_runtime(present: &dyn Fn(&str) -> bool) -> Option<&'static str> {
    RUNTIMES.into_iter().find(|r| present(r))
}

/// What to say when there is no container runtime.
///
/// 7h's exit condition: it **refuses and names what is missing**, rather than running anywhere.
/// The alternative — quietly rehearsing on the host — would be a command whose entire purpose
/// is "not on this machine" doing the thing on this machine.
pub fn no_runtime_refusal() -> String {
    format!(
        "refusing to rehearse: `try` needs a container runtime and found neither {}.\n  \
         The whole point of `try` is to answer \"would this config work on a machine that is \
         not mine\", so running it here would answer a different question and call it the same \
         one.\n  \
         Install one, or use `shall check` to validate the config against this machine.",
        RUNTIMES.join(" nor ")
    )
}

/// What to say when the runtime could not produce the image.
///
/// It says *could not find*, not *does not have*: the same failure covers a daemon that is not
/// running, and a message asserting the image is missing would send someone to build an image
/// they already have. `try` cannot tell the two apart from one exit code, so it does not claim
/// to — it names both and lets the reader look.
pub fn missing_image_refusal(runtime: &str, image: &str) -> String {
    let known: Vec<String> = IMAGES
        .iter()
        .map(|(name, what)| format!("    {}   ({})", name, what))
        .collect();
    format!(
        "refusing to rehearse: `{runtime}` could not find an image named `{image}`.\n  \
         Either it has not been built, or `{runtime}` is installed but not running — \
         `{runtime} info` says which.\n  \
         `try` reuses the integration images, which are built from `docker/integration/`:\n\n\
         {images}\n\n  \
         Build one with:  {runtime} build -f docker/integration/Dockerfile.ubuntu -t {default} .",
        runtime = runtime,
        image = image,
        images = known.join("\n"),
        default = DEFAULT_IMAGE
    )
}

/// The command that runs the rehearsal.
///
/// `--rm` so the container is gone afterwards, `:ro` on the config so the rehearsal cannot
/// write to the repo, and a data directory inside the container so nothing it records survives.
/// The check runs as the container's own `shall`, which is the point: a different machine's
/// binary, backends and installed set.
pub fn argv(runtime: &str, image: &str, config_host_path: &str) -> Vec<String> {
    [
        runtime,
        "run",
        "--rm",
        "-v",
        &format!("{}:/shall-config:ro", config_host_path),
        "-e",
        "SHALL_CONFIG_DIR=/shall-config",
        "-e",
        "SHALL_DATA_DIR=/tmp/shall-try-data",
        "--entrypoint",
        "shall",
        image,
        // `eval`, not `check`. `check` compares the config against the machine, and on a bare
        // container it always finds differences — which U21 makes exit 2, indistinguishable
        // from the exit 2 it uses for a config that does not parse. A rehearsal built on that
        // can never reject anything, and this one did not until a container proved it.
        // `eval` asks only whether the config RESOLVES, which is exactly what `try` wants to
        // know: it exits 0 when it does and non-zero when it does not.
        "eval",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// What the rehearsal's exit code meant.
///
/// `eval` resolves and prints, or it fails — so 0 is the only success, and there is no
/// "differences" case to swallow. This was `check` with 0-or-2 accepted, which made `try`
/// unable to reject anything at all: `check` exits 2 both for "nothing is installed here"
/// (always true in a container) and for "this config does not parse".
pub fn verdict(code: Option<i32>) -> Verdict {
    match code {
        Some(0) => Verdict::Valid,
        Some(c) => Verdict::Rejected(c),
        // Killed by a signal: no exit code, and no evidence the config is fine.
        None => Verdict::Rejected(1),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The config resolved on a clean machine.
    Valid,
    /// It did not, with the container's exit code.
    Rejected(i32),
}

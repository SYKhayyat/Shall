use crate::config::config::SandboxSettings;
use crate::core::{Error, Result};
#[allow(unused_imports)] // `Path` is unused on macOS but used on linux/windows
use std::path::Path;
use std::process::Command;
// `info`, `Write` and `NamedTempFile` are used only by the Windows/macOS sandbox paths.
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use tempfile::NamedTempFile;
#[allow(unused_imports)]
use tracing::{debug, info};

/// Configuration for the declarative sandbox environment.
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    pub allow_network: bool,
    pub allow_home: bool,
    pub allow_write: bool,
    pub custom_mounts: Vec<(String, String)>, // (Source, Target)
    pub custom_read_only_mounts: Vec<(String, String)>,
    pub environment: Vec<(String, String)>,
}

/// Whether a command is actually confined, carried beside the command rather than inferred
/// from the settings that asked for it.
///
/// A caller holding a [`Command`] cannot tell the two apart: an unconfined fallback and a real
/// `bwrap` invocation have the same type. Every way this has gone wrong is a caller reporting a
/// boundary it was never handed one of, so the answer travels with the command and a caller that
/// wants to claim confinement has to hold the variant that grants it.
#[derive(Debug, Clone)]
#[must_use]
pub enum Confinement {
    /// A mechanism is in force, named so a caller can say which one.
    By(&'static str),
    /// No mechanism is in force. Carries the reason, so a caller cannot report the absence
    /// without being able to say why it is absent.
    None { because: String },
}

impl Confinement {
    /// The sentence a user is owed when a command they asked to confine is about to run loose.
    pub fn unconfined_warning(&self) -> Option<String> {
        match self {
            Self::By(_) => None,
            Self::None { because } => Some(format!(
                "confinement was requested but this command will run unconfined: {because}"
            )),
        }
    }
}

pub struct Sandbox;

/// Escaping the two sandbox documents have to do, in one place and outside any `cfg`.
///
/// The `.wsb` writer's escaper used to be a `fn xml` nested inside a `#[cfg(target_os =
/// "windows")]` function, which made it invisible to every test on every other host — so the
/// escaping that protects the XML was the part of the sandbox this repo could not exercise. The
/// macOS profile had no escaping at all: it interpolates a home directory and custom mount paths
/// into a Scheme policy string, where an unbalanced quote ends the string and whatever follows
/// becomes policy. A path is user input, and a path is exactly where a quote can be.
pub mod escape {
    /// XML text and attribute escaping, for Windows Sandbox's `.wsb`.
    pub fn xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// A double-quoted string literal for a macOS sandbox profile (Scheme).
    ///
    /// The profile is a normal Scheme program, so a path containing `"` or `\\` would close the
    /// literal and let the rest of the path be read as policy — a directory named `x") (allow
    /// default) ("` grants everything. Only those two characters need escaping inside a quoted
    /// string.
    pub fn scheme_string(value: &str) -> String {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

impl Sandbox {
    /// The one place that answers "is a mechanism in force, and if not may this proceed".
    ///
    /// `run`, `shell` and `wrap` each used to answer it separately, which is how the `warn!` in
    /// `run` became unreachable: its condition was already folded into the predicate in front of
    /// it. The decision is made once here and carried, so the three cannot disagree and a
    /// mechanism that is missing cannot be spelled the same as one that is present.
    ///
    /// `require_bwrap` and `windows_require_sandbox` outrank `fallback_allowed`: they exist to
    /// refuse exactly the run `fallback_allowed` would permit, so a host that has neither
    /// mechanism nor permission to skip it gets an error rather than a bare command.
    pub async fn decide(settings: &SandboxSettings) -> Result<Confinement> {
        let (mechanism, present, required, knob) = if cfg!(target_os = "linux") {
            (
                "bubblewrap",
                Self::bwrap_available(),
                settings.require_bwrap,
                "sandbox.require_bwrap",
            )
        } else if cfg!(target_os = "macos") {
            (
                "sandbox-exec",
                Self::sandbox_exec_available(),
                settings.macos_require_sandbox,
                "sandbox.macos_require_sandbox",
            )
        } else if cfg!(target_os = "windows") {
            (
                "Windows Sandbox",
                Self::windows_sandbox_feature_enabled().await,
                settings.windows_require_sandbox,
                "sandbox.windows_require_sandbox",
            )
        } else {
            (
                std::env::consts::OS,
                false,
                false,
                "sandbox.fallback_allowed",
            )
        };

        if present {
            return Ok(Confinement::By(mechanism));
        }
        if required {
            return Err(Error::UnsupportedPlatform(format!(
                "`{knob}` is set and `{mechanism}` is not functional on this host."
            )));
        }
        if !settings.fallback_allowed {
            return Err(Error::UnsupportedPlatform(format!(
                "Sandboxing is required by policy but `{mechanism}` is not functional on this host."
            )));
        }
        Ok(Confinement::None {
            because: format!("`{mechanism}` is not available on this host"),
        })
    }

    fn bwrap_available() -> bool {
        crate::core::launch::program_exists("bwrap")
    }

    fn sandbox_exec_available() -> bool {
        #[cfg(target_os = "macos")]
        {
            return crate::core::launch::program_exists("sandbox-exec");
        }
        #[allow(unreachable_code)]
        false
    }

    /// Where the Windows Sandbox host binary lives. Named once because `decide` and
    /// `wrap_windows` asking two different questions about one mechanism is how the verdict and
    /// the command they build came apart.
    #[cfg(target_os = "windows")]
    const WSB_EXE: &'static str = "C:\\Windows\\System32\\WindowsSandbox.exe";

    /// Detects if the Windows Sandbox optional feature is enabled *and* its host binary is on
    /// disk. Both, because the feature can be reported enabled while servicing has not yet laid
    /// the binary down, and `wrap_windows` needs the binary.
    async fn windows_sandbox_feature_enabled() -> bool {
        #[cfg(target_os = "windows")]
        {
            if !Path::new(Self::WSB_EXE).exists() {
                return false;
            }
            let mut command = tokio::process::Command::new("powershell");
            command.args(["-NoProfile", "-NonInteractive", "-Command", "Get-WindowsOptionalFeature -Online -FeatureName 'Containers-DisposableClient' | Select-Object -ExpandProperty State"]);
            // Supervised: `Get-WindowsOptionalFeature` talks to the servicing stack, which is
            // the component most likely on a Windows box to answer in its own time or not at all.
            let output =
                crate::core::supervise::supervised_output(command, "powershell", false).await;

            if let Ok(out) = output {
                return crate::utils::text::sanitize(&String::from_utf8_lossy(&out.stdout))
                    == "Enabled";
            }
        }
        false
    }

    /// Generates a Windows Sandbox (.wsb) configuration file content.
    #[cfg(target_os = "windows")]
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn generate_wsb_config(cmd: &str, args: &[String], config: &SandboxConfig) -> String {
        // Windows Sandbox maps each host folder to the guest user's Desktop under the host
        // folder's basename; the schema has no arbitrary guest-target field. Keep that rule in
        // one helper so the caller can construct a PATH that names the same guest locations.
        fn guest_folder_name(source: &str) -> String {
            let source = source.replace('/', "\\");
            let name = Path::new(&source)
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .unwrap_or("ShallMount");
            if name.contains(':') {
                return name.to_string();
            }
            if source.starts_with(r"C:\") || source.starts_with(r"D:\") {
                return source;
            }
            format!(r"C:\Users\WDAGUtilityAccount\Desktop\{}", name)
        }

        let mut wsb = String::from("<Configuration>\n");

        wsb.push_str("  <VGpu>Disable</VGpu>\n");
        wsb.push_str(&format!(
            "  <Networking>{}</Networking>\n",
            if config.allow_network {
                "Default"
            } else {
                "Disable"
            }
        ));

        wsb.push_str("  <MappedFolders>\n");
        let mut mapped = Vec::new();
        if config.allow_home {
            if let Some(home) = dirs::home_dir() {
                mapped.push((
                    home.to_string_lossy().into_owned(),
                    String::from(r"C:\Users\WDAGUtilityAccount\Desktop\Home"),
                    config.allow_write,
                ));
            }
        }
        mapped.extend(
            config
                .custom_read_only_mounts
                .iter()
                .map(|(src, target)| (src.clone(), guest_folder_name(target), true)),
        );
        mapped.extend(
            config
                .custom_mounts
                .iter()
                .map(|(src, target)| (src.clone(), guest_folder_name(target), false)),
        );
        for (src, target, read_only) in mapped {
            // **A missing mount is said out loud.** The `.wsb` schema has no way to declare a
            // folder it cannot see, so dropping one is the only option — but dropping it in
            // silence means a `custom_mounts` line naming a path that is not there produces a
            // sandbox missing a directory the user asked for, and the run either fails somewhere
            // unrelated or quietly works without it. The caller reads stderr; the file is
            // generated, used, and destroyed.
            if !Path::new(&src).exists() {
                tracing::warn!(
                    "not mapping {}: it does not exist on this host, so the sandbox will not \
                     have it",
                    src
                );
                continue;
            }
            {
                wsb.push_str("    <MappedFolder>\n");
                wsb.push_str(&format!(
                    "      <HostFolder>{}</HostFolder>\n",
                    escape::xml(&src)
                ));
                wsb.push_str(&format!(
                    "      <SandboxFolder>{}</SandboxFolder>\n",
                    escape::xml(&target)
                ));
                wsb.push_str(&format!(
                    "      <ReadOnly>{}</ReadOnly>\n",
                    if read_only { "true" } else { "false" }
                ));
                wsb.push_str("    </MappedFolder>\n");
            }
        }
        wsb.push_str("  </MappedFolders>\n");

        if !config.environment.is_empty() {
            wsb.push_str("  <EnvironmentVariables>\n");
            for (name, value) in &config.environment {
                wsb.push_str(&format!(
                    "    <Variable name=\"{}\" value=\"{}\" />\n",
                    escape::xml(name),
                    escape::xml(value)
                ));
            }
            wsb.push_str("  </EnvironmentVariables>\n");
        }

        // WSB parses this as a command line after XML decoding. Keep arguments as separate
        // quoted tokens so an argument containing whitespace cannot become a second command;
        // XML escaping above handles the serialization layer independently.
        let full_cmd = std::iter::once(cmd)
            .chain(args.iter().map(String::as_str))
            .map(|arg| {
                if arg.chars().any(char::is_whitespace) || arg.contains(['&', '<', '>', '"']) {
                    format!("\\\"{}\\\"", arg.replace('"', "\\\""))
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        wsb.push_str("  <LogonCommand>\n");
        wsb.push_str(&format!(
            "    <Command>{}</Command>\n",
            escape::xml(&full_cmd)
        ));
        wsb.push_str("  </LogonCommand>\n");

        wsb.push_str("</Configuration>");
        wsb
    }

    #[cfg(target_os = "macos")]
    fn generate_macos_profile(config: &SandboxConfig) -> String {
        let mut profile = String::from("(version 1)\n(deny default)\n");
        profile.push_str("(allow sysctl-read)\n(allow signal (target self))\n(allow process-fork)\n(allow process-exec)\n");

        let ro_paths = [
            "/usr/lib",
            "/usr/share",
            "/System/Library",
            "/Library/Preferences",
            "/bin",
            "/usr/bin",
        ];
        for path in ro_paths {
            let path = escape::scheme_string(path);
            profile.push_str(&format!("(allow file-read* (subpath {path}))\n"));
        }

        if config.allow_network {
            profile
                .push_str("(allow network*)\n(allow file-read* (literal \"/etc/resolv.conf\"))\n");
        }

        if config.allow_home {
            if let Some(home) = dirs::home_dir() {
                let path = escape::scheme_string(&home.to_string_lossy());
                profile.push_str(&format!("(allow file-read* (subpath {path}))\n"));
                if config.allow_write {
                    profile.push_str(&format!("(allow file-write* (subpath {path}))\n"));
                }
            }
        }

        for (src, _) in &config.custom_read_only_mounts {
            let src = escape::scheme_string(src);
            profile.push_str(&format!("(allow file-read* (subpath {src}))\n"));
        }
        for (src, _) in &config.custom_mounts {
            let src = escape::scheme_string(src);
            profile.push_str(&format!("(allow file-read* (subpath {src}))\n"));
            profile.push_str(&format!("(allow file-write* (subpath {src}))\n"));
        }

        profile
    }

    /// Build the command `decided` describes.
    ///
    /// The verdict is passed in rather than recomputed: recomputing it here is what let the
    /// decision differ between the caller that reports to the user and the code that builds the
    /// process. An unconfined verdict yields the bare command and nothing else — in particular it
    /// does not claim an isolation it is not building.
    pub fn wrap(
        cmd: &str,
        args: &[String],
        config: &SandboxConfig,
        settings: &SandboxSettings,
        decided: &Confinement,
    ) -> Result<Wrapped> {
        let _ = (config, settings);
        if let Confinement::None { .. } = decided {
            // Windows keeps its below-normal-priority launch here rather than in `By`: it is
            // a real scheduling courtesy, and it is NOT a sandbox — `start /low` sets a
            // priority class, nothing about the token, so naming it confinement would be
            // the claim this type exists to stop. The verdict stays `None` with its reason.
            #[cfg(target_os = "windows")]
            {
                return Ok(Wrapped::bare(Self::low_integrity_windows(
                    cmd, args, config,
                )));
            }
            #[allow(unreachable_code)]
            {
                let mut bare = Command::new(cmd);
                bare.args(args);
                return Ok(Wrapped::bare(bare));
            }
        }

        #[cfg(target_os = "linux")]
        {
            let bare = Self::wrap_linux(cmd, args, config)?;
            return Ok(Wrapped::bare(bare));
        }

        #[cfg(target_os = "macos")]
        {
            let bare = Self::wrap_macos(cmd, args, config, settings)?;
            return Ok(Wrapped::bare(bare));
        }

        #[cfg(target_os = "windows")]
        {
            return Self::wrap_windows(cmd, args, config);
        }

        #[allow(unreachable_code)]
        Err(Error::UnsupportedPlatform(format!(
            "Sandboxing not supported on {}",
            std::env::consts::OS
        )))
    }

    #[cfg(target_os = "linux")]
    fn wrap_linux(cmd: &str, args: &[String], config: &SandboxConfig) -> Result<Command> {
        // Reached only for a `Confinement::By` verdict. A mechanism that has gone missing between
        // the decision and here is an error rather than a fallback: the caller has already told
        // the user this command is confined.
        if !Self::bwrap_available() {
            return Err(Error::UnsupportedPlatform(
                "bubblewrap (bwrap) was available when this run was planned and is not now.".into(),
            ));
        }

        let mut bwrap = Command::new("bwrap");
        bwrap.arg("--unshare-all");
        if config.allow_network {
            bwrap.arg("--share-net");
        }

        let ro_paths = ["/usr", "/bin", "/lib", "/lib64", "/etc/alternatives"];
        for path in ro_paths {
            if Path::new(path).exists() {
                bwrap.arg("--ro-bind").arg(path).arg(path);
            }
        }

        bwrap
            .arg("--dev")
            .arg("/dev")
            .arg("--proc")
            .arg("/proc")
            .arg("--tmpfs")
            .arg("/tmp");

        if config.allow_home {
            if let Some(home) = dirs::home_dir() {
                if config.allow_write {
                    bwrap.arg("--bind").arg(&home).arg(&home);
                } else {
                    bwrap.arg("--ro-bind").arg(&home).arg(&home);
                }
            }
        }

        for (src, target) in &config.custom_read_only_mounts {
            if Path::new(src).exists() {
                bwrap.arg("--ro-bind").arg(src).arg(target);
            }
        }
        for (src, target) in &config.custom_mounts {
            if Path::new(src).exists() {
                bwrap.arg("--bind").arg(src).arg(target);
            }
        }

        // **The environment starts EMPTY.** Without `--clearenv`, bwrap inherits the whole
        // parent environment additively — `--setenv` on top of it — and every cloud token,
        // proxy credential and API key Shall itself holds crosses into the "confined"
        // process untouched.
        bwrap.arg("--clearenv");

        for (key, value) in &config.environment {
            bwrap.arg("--setenv").arg(key).arg(value);
        }

        bwrap.arg("--").arg(cmd).args(args);
        Ok(bwrap)
    }

    #[cfg(target_os = "macos")]
    fn wrap_macos(
        cmd: &str,
        args: &[String],
        config: &SandboxConfig,
        settings: &SandboxSettings,
    ) -> Result<Command> {
        // As `wrap_linux`: reached only for a `Confinement::By` verdict, so a mechanism that has
        // gone missing since is an error rather than a silent unconfined run.
        if !Self::sandbox_exec_available() {
            return Err(Error::UnsupportedPlatform(
                "sandbox-exec was available when this run was planned and is not now.".into(),
            ));
        }

        let profile = if let Some(ref path) = settings.macos_profile_template {
            std::fs::read_to_string(path)
                .map_err(|e| Error::Config(format!("Failed to read custom macOS profile: {}", e)))?
        } else {
            Self::generate_macos_profile(config)
        };

        let mut sandbox_cmd = Command::new("sandbox-exec");
        sandbox_cmd.arg("-p").arg(profile);
        // **The environment starts EMPTY** (same ruling as the bwrap path): `env` here is
        // additive onto what Shall itself holds, and a "confined" tool that can read
        // Shall's cloud tokens is not confined in any sense that matters.
        sandbox_cmd.env_clear();
        for (key, value) in &config.environment {
            sandbox_cmd.env(key, value);
        }
        sandbox_cmd.arg(cmd).args(args);
        Ok(sandbox_cmd)
    }

    #[cfg(target_os = "windows")]
    fn wrap_windows(cmd: &str, args: &[String], config: &SandboxConfig) -> Result<Wrapped> {
        // As `wrap_linux`: reached only for a `Confinement::By` verdict.
        if !Path::new(Self::WSB_EXE).exists() {
            return Err(Error::UnsupportedPlatform(
                "Windows Sandbox was available when this run was planned and is not now.".into(),
            ));
        }

        info!("Sandboxing (Windows): Launching hardware-isolated environment (.wsb)");
        let wsb_content = Self::generate_wsb_config(cmd, args, config);
        let mut tmp_file = NamedTempFile::new().map_err(Error::from)?;
        tmp_file
            .write_all(wsb_content.as_bytes())
            .map_err(Error::from)?;
        tmp_file.flush().map_err(Error::from)?;
        let mut command = Command::new(Self::WSB_EXE);
        command.arg(tmp_file.path());
        // The `.wsb` rides beside the command and is deleted when the [`Wrapped`] is. It used
        // to be a local here, deleted on return — before the caller had spawned anything, so
        // Windows Sandbox was handed a path to a file that no longer existed and the
        // "confined" run ran bare.
        Ok(Wrapped::with_config_file(command, tmp_file))
    }

    /// The `cmd` argv for Windows' unconfined path, and **it waits for the child** (#133).
    ///
    /// BELOW-NORMAL PRIORITY is a scheduling courtesy and not an integrity reduction —
    /// `start /low` sets the priority class and touches nothing about the token. Reached only
    /// for a `Confinement::None` verdict, whose `because` already says no mechanism is in
    /// force; this exists so the run still starts politely, and this doc claims exactly that
    /// much.
    ///
    /// The empty `""` after `start` is its window-title slot: without it, `start` reads the
    /// first QUOTED argument as the title and runs nothing.
    ///
    /// **`/wait` is not decoration, and its absence was a defect for as long as this line has
    /// existed.** `start` returns as soon as the process is created, so `cmd /c` exited, and
    /// `Sandbox::run`'s `status()` reported `start`'s status — almost always 0 — before the
    /// wrapped command had done anything. A failing program was reported as 0; its output
    /// arrived after Shall had printed its own exit line and returned to the prompt, so a
    /// pipe got a truncated answer; and `start` detached from the job tree, so `Stopping`
    /// could not reach it. That is the whole Windows half of "spawn a child and wait for it",
    /// on the path a Windows host **without** Windows Sandbox takes — which is most of them.
    ///
    /// The argv is built and returned as data, rather than inline where the `Command` is
    /// assembled, so a test can assert it on a host that cannot run it: the local chain
    /// type-checks `cfg(windows)` and executes none of it, which is how the flag went missing.
    /// `tests/app/sandbox/the_unconfined_windows_launch_waits_for_its_child.rs` compares the
    /// whole line, because a test that looks for one word passes on an argv that runs nothing.
    pub fn windows_unconfined_argv(cmd: &str, args: &[String]) -> Vec<String> {
        let mut argv = vec![
            "/c".to_string(),
            "start".to_string(),
            String::new(),
            "/low".to_string(),
            "/wait".to_string(),
            "/b".to_string(),
            cmd.to_string(),
        ];
        argv.extend(args.iter().cloned());
        argv
    }

    /// Windows' unconfined launch: the argv [`Sandbox::windows_unconfined_argv`] explains,
    /// spawned by `cmd` with an empty environment.
    #[cfg(target_os = "windows")]
    fn low_integrity_windows(cmd: &str, args: &[String], config: &SandboxConfig) -> Command {
        let mut command = Command::new("cmd");
        command.args(Self::windows_unconfined_argv(cmd, args));
        // **The environment starts EMPTY** (same ruling as the bwrap path): additive env
        // onto Shall's own means every token Shall holds rides into the process.
        command.env_clear();
        for (key, value) in &config.environment {
            command.env(key, value);
        }
        if !config.allow_home {
            command.env("USERPROFILE", "C:\\Users\\Public");
        }
        command
    }

    /// Executes a command under the confinement `decided` describes.
    pub fn run(
        cmd: &str,
        args: &[String],
        config: &SandboxConfig,
        settings: &SandboxSettings,
        decided: &Confinement,
    ) -> Result<std::process::ExitStatus> {
        let mut wrapped = Self::wrap(cmd, args, config, settings, decided)?;
        wrapped.command.status().map_err(Error::from)
    }
}

/// A command under confinement, and whatever has to outlive it.
///
/// The command alone was not enough to hand back: Windows Sandbox reads its `.wsb` file at
/// launch, so the temp file holding it must live until the process has run — it used to be a
/// local in the wrapper, deleted before anything was spawned, and the confinement silently
/// did not happen. The keep-alive travels with the command so no caller can hold one without
/// the other.
pub struct Wrapped {
    pub command: Command,
    /// Deleted on drop — which is the right time only because this struct outlives every
    /// `status`/`wait` its command is handed to. `None` when nothing extra is held.
    _keepalive: Option<NamedTempFile>,
}

impl Wrapped {
    fn bare(command: Command) -> Self {
        Self {
            command,
            _keepalive: None,
        }
    }

    #[cfg(target_os = "windows")]
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn with_config_file(command: Command, keepalive: NamedTempFile) -> Self {
        Self {
            command,
            _keepalive: Some(keepalive),
        }
    }
}

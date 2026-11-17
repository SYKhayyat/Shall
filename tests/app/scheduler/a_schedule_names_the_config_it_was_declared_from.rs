//! **A task fires against the config that declared it** (#192).
//!
//! Every renderer wrote `<shall_bin> <command>` and nothing else, so a schedule declared by a run
//! with a non-default root — `shall --config-dir X`, `$SHALL_CONFIG_DIR`, a second checkout —
//! registered fine and then fired `shall sync` against the **default** config and data dir. That
//! is a different machine description from the one that declared it, and nothing said so: the
//! task looks correct in `schtasks /query` and in the unit file, and does the wrong thing an hour
//! later.
//!
//! The roots are written into the command line only when they are not the defaults, so an
//! ordinary task keeps exactly the spelling it had and the two spellings cannot be confused for
//! one another.

use super::*;
use shall::app::scheduler::Roots;

/// The flags a non-default root produces, in all three renderers.
#[test]
fn every_renderer_names_a_non_default_config_root() {
    let config = PathBuf::from("/srv/other-checkout/shall-config");
    let data = PathBuf::from("/srv/other-checkout/shall-state");
    let target = here().with_roots(Roots {
        config: config.clone(),
        data: data.clone(),
    });
    let bin = Path::new("/usr/local/bin/shall");

    let unit = LinuxSystemdProvisioner::service_unit(&cfg("0 2 * * *"), bin, &target);
    assert!(
        unit.contains("--config-dir /srv/other-checkout/shall-config"),
        "the systemd unit does not name the config it was declared from:\n{unit}"
    );
    assert!(
        unit.contains("--data-dir /srv/other-checkout/shall-state"),
        "the systemd unit does not name the state it was declared from:\n{unit}"
    );
    assert!(
        unit.contains("ExecStart=/usr/local/bin/shall --config-dir"),
        "the flags have to sit between the binary and the verb, not after it:\n{unit}"
    );

    let plist = MacLaunchdProvisioner.plist(&cfg("0 2 * * *"), bin, &target);
    // launchd hands `ProgramArguments` to `execve`, so the root is its own `<string>` and is not
    // quoted — a quoted element would arrive with its quotes.
    assert!(
        plist.contains(
            "<string>--config-dir</string><string>/srv/other-checkout/shall-config</string>"
        ),
        "the plist does not pass the config root as its own argv element:\n{plist}"
    );

    let line = WindowsTaskProvisioner::command_line(&cfg("0 2 * * *"), bin, &target);
    assert!(
        line.contains("--config-dir /srv/other-checkout/shall-config"),
        "the schtasks command line does not name the config it was declared from: {line}"
    );
    assert!(
        line.starts_with("C:\\") || line.starts_with("/usr/local/bin/shall"),
        "the binary has to come first: {line}"
    );
}

/// **And the defaults are spelled as they always were.** A task registered by an ordinary run must
/// not start carrying `--config-dir ~/.config/shall`, because that line is a promise about a path
/// that could move — and because a change with no defect behind it is a change nobody asked for.
#[test]
fn a_default_root_writes_no_flag_at_all() {
    let bin = Path::new("/usr/local/bin/shall");
    let defaults = Roots {
        config: shall::utils::safe_config_dir(),
        data: shall::utils::safe_data_dir(),
    };
    let target = here().with_roots(defaults);

    let unit = LinuxSystemdProvisioner::service_unit(&cfg("0 2 * * *"), bin, &target);
    assert!(
        unit.contains("ExecStart=/usr/local/bin/shall sync\n"),
        "a default-root task changed its spelling:\n{unit}"
    );
    assert!(
        !unit.contains("--config-dir"),
        "a default root must not be written out:\n{unit}"
    );

    let line = WindowsTaskProvisioner::command_line(&cfg("0 2 * * *"), bin, &target);
    assert_eq!(line, "/usr/local/bin/shall sync");
}

/// A root with a space in it is quoted on the two renderers that write a command *line*, and is
/// not quoted on the one that writes an argv array. A path with a space is ordinary on Windows.
#[test]
fn a_root_with_a_space_is_quoted_only_where_a_line_is_parsed() {
    let spaced = PathBuf::from("/srv/two words/config");
    let target = here().with_roots(Roots {
        config: spaced.clone(),
        data: PathBuf::new(),
    });
    let bin = Path::new("/usr/local/bin/shall");

    let unit = LinuxSystemdProvisioner::service_unit(&cfg("0 2 * * *"), bin, &target);
    assert!(
        unit.contains("--config-dir \"/srv/two words/config\""),
        "systemd splits ExecStart on whitespace, so the path has to be quoted:\n{unit}"
    );

    let plist = MacLaunchdProvisioner.plist(&cfg("0 2 * * *"), bin, &target);
    assert!(
        plist.contains("<string>/srv/two words/config</string>"),
        "launchd takes an argv element, so the path arrives whole and unquoted:\n{plist}"
    );
    assert!(
        !plist.contains("\"/srv/two words/config\""),
        "a quoted argv element would arrive with its quotes:\n{plist}"
    );
}

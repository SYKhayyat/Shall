use super::*;

/// **`xbps-query -l` lists the package database, not the installed set.**
///
/// Every row opens with a two-character state word, and only `ii` means the software is on the
/// machine: `uu` is `XBPS_PKG_STATE_UNPACKED` — registered, files extracted, the configure half
/// never run — `hr` is a removal that stopped part-way, and `??` is a state this xbps does not
/// know. The old reader took the first token that parsed as a pkgver and never looked at the
/// word, so all four read as installed.
///
/// **What that cost is the crash section's defect.** `xbps-install` registers a package before
/// it configures it (`lib/transaction_commit.c`: unpack, register, configure), so a machine
/// killed in that window comes back with the package in the database and not usable. `sync`
/// asked the listing, the listing said yes, and the plan never named the package — the machine
/// stayed missing a declared package while every run reported convergence.
///
/// The same shape as the `dpkg` `${db:Status-Status}` fix in `parsers/apt.rs`, which is where the
/// vocabulary table comes from; a row read correctly and dropped for its state is an *answer*,
/// so it must not be reported as an unrecognised shape.
#[test]
fn a_state_word_decides_whether_a_row_is_installed() {
    let out = "\
ii bash-5.2.15_2            The GNU Bourne Again Shell
uu pv-1.8.1_1               Pipe Viewer
hr ncdu-1.19_1              A ncurses disk usage viewer
?? mystery-0.1_1            Registered by a state this xbps does not know
";
    let pkgs = parse_xbps_list(out).expect("real xbps -l output");
    assert_eq!(
        pkgs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["bash"],
        "a package that is registered but not installed was reported as installed"
    );

    // A machine where every known package is in a half state is a real machine: the answer is
    // an empty installed set, not a refusal to read the listing.
    let all_half = "uu pv-1.8.1_1  Pipe Viewer\nhr ncdu-1.19_1  disk usage\n";
    assert_eq!(
        parse_xbps_list(all_half)
            .expect("a listing of only non-installed rows is still a listing")
            .len(),
        0
    );

    // Junk still has to be readable as junk — a row with neither a state word nor a pkgver is
    // the manager's error output, and an empty answer from it is the dangerous direction.
    assert!(parse_xbps_list("E: repository is unavailable\n").is_err());
}

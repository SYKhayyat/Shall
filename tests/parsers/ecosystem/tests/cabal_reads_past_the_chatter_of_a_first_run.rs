use super::*;

/// Real bytes, and the whole reason `cabal_list` exists: a container's first `cabal list`
/// writes three lines of configuration chatter to **stdout** ahead of the packages.
///
/// **`Cabal 3.10.3.0` stays.** The 2026-08-23 audit called this line the tool's first-run
/// banner and the original expectation a phantom; the builtin row's captured fixture
/// (docker haskell:9.6) says otherwise — `cabal list --installed` really does list the
/// global `Cabal` library, textually identical to any banner. The repo's own capture wins
/// over an audit claim: the chatter lines fail the digit-version rule and Cabal is data.
#[test]
fn cabal_reads_past_the_chatter_of_a_first_run() {
    let out = "Config file path source is default config file.\n\
               Config file not found: /root/.config/cabal/config\n\
               Writing default configuration to /root/.config/cabal/config\n\
               Cabal 3.10.3.0\n\
               array 0.5.8.0\n";
    let pkgs = cabal_list(out, "cabal").expect("this is what haskell:9.6 printed");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "Cabal");
    assert_eq!(pkgs[1].name, "array");
}

use super::*;

/// Q5. `@unverified` is legal wherever *something* verifies bytes and the line can say
/// "not here" — Shall's own checksum on a download, and helm's plugin signature.
#[test]
fn unverified_is_legal_on_every_backend_that_verifies_something() {
    for line in [
        "web:https://example.com/tool@unverified",
        "appimage:https://example.com/x.AppImage@unverified",
        "github:sharkdp/fd@unverified",
        "helm:diff@url=https://github.com/databus23/helm-diff,unverified",
    ] {
        assert_eq!(
            options_of(line).one("unverified"),
            Some("true"),
            "`@unverified` was not accepted on `{}`",
            line
        );
    }
}

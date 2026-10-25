use super::*;

/// D11: the default order is versioned, and this test is the tripwire. It pins the exact
/// order for the three host shapes to `DEFAULT_ORDER_VERSION`. Changing what
/// `detected_default` returns fails this test until the constant is bumped — which is the
/// point: a change to the default is a deliberate, visible event, not a silent one an
/// upgrade slips in.
#[test]
fn the_default_order_is_pinned_to_its_version() {
    assert_eq!(
        FormatOrder::DEFAULT_ORDER_VERSION,
        1,
        "bump the version WITH the order"
    );

    let debian = FormatOrder::detected_default("linux", Some("debian"));
    assert_eq!(
        debian.as_slice(),
        &[
            Format::Deb,
            Format::AppImage,
            Format::Tarball,
            Format::Binary
        ]
    );
    let windows = FormatOrder::detected_default("windows", None);
    assert_eq!(windows.as_slice(), &[Format::Exe, Format::Msi, Format::Zip]);
    let macos = FormatOrder::detected_default("macos", None);
    assert_eq!(
        macos.as_slice(),
        &[
            Format::Dmg,
            Format::Pkg,
            Format::Tarball,
            Format::Zip,
            Format::Binary
        ]
    );
}

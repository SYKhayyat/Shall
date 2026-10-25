//! The escape hatch the stricter rule needs: a project shipping one bare executable for one
//! platform is installed by naming it, because naming it is the claim.

use super::*;

#[test]
fn naming_an_asset_exactly_installs_it_even_when_the_name_says_nothing() {
    let assets = vec![Asset::new("mytool", "https://example.test/mytool")];
    let platform = Platform::new("linux", "x86_64");
    let formats = FormatOrder::detected_default("linux", None);
    let pattern = AssetPattern::parse("mytool").unwrap();
    let picked = select(
        &Request {
            package: "acme/mytool",
            release: "fixture",
            platform: &platform,
            formats: &formats,
            pattern: Some(&pattern),
        },
        &assets,
    )
    .expect("an exactly-named asset installs");
    assert_eq!(picked.picks[0].format, Format::Binary);

    // A glob is not a claim about the platform, so it does not open the same door.
    let glob = AssetPattern::parse("my*").unwrap();
    assert!(select(
        &Request {
            package: "acme/mytool",
            release: "fixture",
            platform: &platform,
            formats: &formats,
            pattern: Some(&glob),
        },
        &assets,
    )
    .is_err());
}

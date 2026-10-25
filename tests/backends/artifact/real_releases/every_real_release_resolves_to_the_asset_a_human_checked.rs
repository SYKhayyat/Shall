use super::*;

#[test]
fn every_real_release_resolves_to_the_asset_a_human_checked() {
    for b in blocks() {
        for (os, arch, want) in &b.expected {
            let platform = Platform::new(os.clone(), arch.clone());
            let formats = FormatOrder::detected_default(os, family_of(os));
            let request = Request {
                package: &b.repo,
                release: "fixture",
                platform: &platform,
                formats: &formats,
                pattern: None,
            };
            let got = match select(&request, &b.assets) {
                Ok(sel) => sel.picks[0].asset.name.clone(),
                Err(_) => "none".to_string(),
            };
            assert_eq!(&got, want, "{} on {}/{}", b.repo, os, arch);
        }
    }
}

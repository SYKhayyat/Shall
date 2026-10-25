use super::*;

/// **The gate F-7 is really about.** `Format` offered `.tar.zst` and `.txz` as tarballs and
/// nothing could open either, so the selector and the extractor disagreed in silence: the
/// asset was chosen, downloaded, `fs::copy`d whole into the destination, searched for an
/// executable, found to have none — and the install reported success having deployed
/// nothing. A check drawn around `extract_archive`'s own four suffixes could never have
/// seen it, because the missing suffixes were in the *other* list.
#[test]
fn every_archive_the_selector_offers_has_an_opener() {
    let offered: Vec<&str> = Format::ALL
        .into_iter()
        .filter(|f| f.is_archive())
        .flat_map(|f| f.suffixes().iter().copied())
        .collect();

    // The instrument first: a scan that enumerates nothing passes silently, and this one
    // is reading a table it could just as easily read as empty.
    assert!(
        offered.len() >= 8,
        "the scan found {} archive suffixes, which is fewer than the tarball list alone \
         holds — it is reading the wrong thing",
        offered.len()
    );
    assert!(offered.contains(&".tar.zst"), "{:?}", offered);

    for suffix in offered {
        assert!(
            Format::opener_for(&format!("thing{}", suffix)).is_some(),
            "`{}` is offered as an archive and nothing can open it — which is the exact \
             shape that made a `.tar.zst` install report success having deployed nothing",
            suffix
        );
    }
}

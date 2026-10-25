use super::*;

/// **A URL tail is not a file name.** The query string and the fragment come with it, and
/// CDN-signed asset URLs carry one as a matter of course: on Windows `?` is illegal so the
/// create failed with an I/O error naming nothing, and on Unix it succeeded and put the
/// token in the file name and then in the PATH link name.
#[test]
fn a_url_names_its_last_path_segment_and_nothing_else() {
    for (url, want) in [
        ("https://host/tool.AppImage", "tool.AppImage"),
        ("https://host/tool.AppImage?token=abc", "tool.AppImage"),
        ("https://host/tool.AppImage#frag", "tool.AppImage"),
        ("https://host/a/b/rg.tar.gz?x=1#y", "rg.tar.gz"),
        ("https://host/p%2Fq", "p%2Fq"),
    ] {
        assert_eq!(url_filename(url).unwrap(), want, "{url}");
    }
}

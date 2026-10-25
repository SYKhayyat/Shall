use futures::StreamExt;
pub use shall::core::download::*;
pub use shall::core::download::{human_bytes, part_path, stage_capped_to};
pub use shall::core::{Error, PackageSpec, Result};

fn spec(opts: &[(&str, &str)]) -> PackageSpec {
    PackageSpec {
        name: "http://example.invalid/x".into(),
        backend: "web".into(),
        options: opts.iter().map(|(k, v)| (*k, *v)).collect(),
        requires: vec![],
        present: true,
    }
}
fn undeclared(body: &'static str) -> reqwest::Response {
    let stream = futures::stream::once(async move { Ok::<_, std::io::Error>(body.as_bytes()) });
    reqwest::Response::from(
        http::Response::builder()
            .body(reqwest::Body::wrap_stream(stream))
            .expect("a response with a streamed body"),
    )
}
fn breaks_after(prefix: &'static str) -> reqwest::Response {
    let stream = futures::stream::once(async move { Ok::<_, std::io::Error>(prefix.as_bytes()) })
        .chain(futures::stream::once(async {
            Err::<&[u8], _>(std::io::Error::other("the connection went away"))
        }));
    reqwest::Response::from(
        http::Response::builder()
            .body(reqwest::Body::wrap_stream(stream))
            .expect("a response with a streamed body"),
    )
}
fn declares(len: u64, body: &'static str) -> reqwest::Response {
    let stream = futures::stream::once(async move { Ok::<_, std::io::Error>(body.as_bytes()) });
    reqwest::Response::from(
        http::Response::builder()
            .header(http::header::CONTENT_LENGTH, len)
            .body(reqwest::Body::wrap_stream(stream))
            .expect("a response with a streamed body"),
    )
}
mod a_body_over_the_ceiling_is_refused_and_leaves_nothing_behind;
mod a_body_that_matches_its_declared_length_is_accepted;
mod a_body_under_the_ceiling_is_written_whole;
mod a_download_with_no_checksum_is_refused;
mod a_failed_download_leaves_the_working_artifact_untouched;
mod a_failed_transfer_leaves_nothing_behind;
mod a_size_is_reported_the_way_a_person_reads_one;
mod a_staged_body_dropped_without_a_commit_takes_nothing_else_with_it;
mod a_staged_file_from_a_killed_run_is_removed_before_the_next_attempt;
mod allowing_http_does_not_also_drop_the_checksum;
mod an_unset_flag_is_not_a_set_one;
mod no_ceiling_writes_anything;
mod plain_http_is_refused_and_the_flag_is_what_allows_it;

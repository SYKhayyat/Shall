use super::*;

#[test]
fn remove_line_takes_the_whole_block_and_leaves_its_neighbours() {
    let body = "schedule:nightly {\n  cron = 0 2 * * *\n  run  = sync\n}\n\
                schedule:weekly {\n  cron = @weekly\n  run  = clean\n}\n";
    let out = remove_line(body, "nightly").unwrap();
    assert!(!out.contains("nightly"));
    assert!(out.contains("schedule:weekly {"));
    assert!(out.contains("run  = clean"));
    assert!(remove_line(&out, "nosuch").is_none());
}

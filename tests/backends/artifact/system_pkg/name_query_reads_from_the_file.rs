use super::*;

#[test]
fn name_query_reads_from_the_file() {
    let argv = query_name_argv(Format::Deb, &deb()).unwrap();
    assert_eq!(argv[0], "dpkg-deb");
    assert!(argv.contains(&"Package".to_string()));
    let argv = query_name_argv(Format::Rpm, &rpm()).unwrap();
    assert_eq!(argv[0], "rpm");
    assert!(argv.contains(&"%{NAME}".to_string()));
}

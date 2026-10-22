use super::*;

#[test]
fn use_parses_module_arguments() {
    // U32: `use workstation(user=shaul, gpu=nvidia)`.
    assert_eq!(
        p("use workstation(user=shaul, gpu=nvidia)").unwrap(),
        Statement::Use(
            Reference::Module("workstation".into()),
            vec![
                ("user".into(), "shaul".into()),
                ("gpu".into(), "nvidia".into())
            ]
        )
    );
}

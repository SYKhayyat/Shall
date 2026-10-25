use super::*;

#[test]
fn every_statement_form_parses() {
    let out = stmts(
        "ripgrep\n\
         apt:curl\n\
         apt:re:^fonts-\n\
         absent:apt:libreoffice\n\
         repo:apt:ppa:deadsnakes/ppa\n\
         shim:jq@source=cargo:jq\n\
         service:nginx@enabled=true\n\
         link:/home/me/.vimrc\n\
         use editors\n",
    );
    assert_eq!(out.len(), 9);
    assert!(matches!(out[3], Statement::Absent(_)));
    assert!(matches!(out[4], Statement::Repo { .. }));
    assert!(matches!(out[8], Statement::Use(Reference::Module(_), _)));
}

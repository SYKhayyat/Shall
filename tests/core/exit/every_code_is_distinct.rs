use shall::core::exit::Exit;

#[test]
fn every_code_is_distinct() {
    let mut seen = Vec::new();
    for e in Exit::ALL {
        assert!(!seen.contains(&e.code()), "{:?} reuses a code", e);
        seen.push(e.code());
    }
}

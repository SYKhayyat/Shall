use super::*;

/// Owner ruling: writing the scope that is already the default is accepted, not refused as
/// redundant. A configuration may state a thing it would also get for free — saying it out
/// loud is how a reader learns the answer without going to look it up.
#[test]
fn writing_the_default_scope_is_not_an_error() {
    assert!(pv("shim:rg@scope=user").is_ok());
    assert!(pv("link:./f@target=~/.f,scope=user").is_ok());
}

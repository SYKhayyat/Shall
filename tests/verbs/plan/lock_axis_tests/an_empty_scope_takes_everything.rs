use super::*;

/// No names is every key — a bare `lock versions` pins everything, as it always did.
#[test]
fn an_empty_scope_takes_everything() {
    assert!(scoped_by("apt:curl", &[]));
    assert!(scoped_by("after_install:nginx", &[]));
}

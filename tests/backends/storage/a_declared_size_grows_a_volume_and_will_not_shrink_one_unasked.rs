use super::*;

/// Q19's ruling, in the one function that carries it. A volume already the declared size is
/// left alone; a bigger declaration grows it; a smaller one is refused unless the line says
/// otherwise, and the refusal names both sizes so the reader can tell which way it went.
#[test]
fn a_declared_size_grows_a_volume_and_will_not_shrink_one_unasked() {
    let ten = 10 * (1 << 30);

    // Already right: nothing to run. Including when the unit differs from the one on disk —
    // a comparison by string here is the "a change on every sync, for ever" bug.
    assert!(resize_plan("vg0", "data", "10G", ten, false)
        .unwrap()
        .is_none());
    assert!(resize_plan("vg0", "data", "10240M", ten, false)
        .unwrap()
        .is_none());

    // Growing needs no permission: the space was not being used by anything.
    let (program, args) = resize_plan("vg0", "data", "20G", ten, false)
        .unwrap()
        .expect("a bigger declaration is a change");
    assert_eq!(program, "lvextend");
    assert_eq!(args, vec!["--resizefs", "-L", "20G", "vg0/data"]);

    // Shrinking without the flag is refused, and the message carries both sizes and the way
    // out — an error that says "no" without saying what to do is a puzzle.
    let err = resize_plan("vg0", "data", "5G", ten, false)
        .expect_err("shrinking unasked must be refused")
        .to_string();
    assert!(err.contains("10G"), "{}", err);
    assert!(err.contains("5G"), "{}", err);
    assert!(err.contains("@allow_shrink=true"), "{}", err);

    // With the flag it runs — and it runs `--resizefs`, which shrinks the filesystem first.
    // A bare `lvreduce` here would be the data loss the flag was supposed to make
    // deliberate rather than the resize it was supposed to permit.
    let (program, args) = resize_plan("vg0", "data", "5G", ten, true)
        .unwrap()
        .expect("with the flag, a smaller declaration is a change");
    assert_eq!(program, "lvreduce");
    assert_eq!(
        args,
        vec!["--resizefs", "--yes", "-L", "5G", "vg0/data"],
        "the filesystem shrinks before the volume, or this is truncation"
    );

    // The flag is `true` and nothing else — a typo must not read as permission. (The
    // decision is made from a bool here; `install` is where the string is compared, and
    // this is the assertion that anything but `true` never reaches the shrink branch.)
    assert!(resize_plan("vg0", "data", "5G", ten, false).is_err());
}

use super::*;

#[test]
fn the_floor_drops_a_nameless_row_and_says_which_field() {
    assert_eq!(row("", None).unusable(), Some("it has no `name`"));
    assert_eq!(row("   ", None).unusable(), Some("it has no `name`"));
    assert_eq!(row("ufw", None).unusable(), None);

    let broken = Row {
        name: "ufw".into(),
        os: None,
        broken: Some("it cannot both open and close a port"),
        live: None,
    };
    assert_eq!(
        broken.unusable(),
        Some("it cannot both open and close a port"),
        "a row's own reason must survive the shared floor"
    );
}

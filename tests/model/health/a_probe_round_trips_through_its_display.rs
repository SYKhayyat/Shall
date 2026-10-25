use super::*;

#[test]
fn a_probe_round_trips_through_its_display() {
    for written in ["port:8080", "systemctl is-active nginx"] {
        let p = Probe::parse(written).unwrap();
        assert_eq!(p.to_string(), written);
    }
}

use super::*;

#[test]
fn the_snapshot_provider_schema_parses() {
    let toml = r#"
[[snapshot]]
name = "lvm"
detect = "lvcreate"
source = "vg0/root"
create = ["lvcreate", "--snapshot", "--name", "{id}", "{source}"]
list = ["lvs", "--noheadings", "-o", "lv_name"]
delete = ["lvremove", "-y", "{id}"]
restore = ["lvconvert", "--merge", "{id}"]
restores_running_system = true
list_pattern = '(shall_\S+)'
"#;
    let file: SnapshotProviderFile = toml::from_str(toml).unwrap();
    assert_eq!(file.snapshot.len(), 1);
    assert!(file.snapshot[0].is_live());
    assert!(file.snapshot[0].unusable().is_none());
}

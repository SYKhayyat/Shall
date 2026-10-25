use super::*;

#[test]
fn history_reports_only_what_was_asked_for() {
    // Verbatim from `conda env export -n base --from-history --json` on the tools test
    // image, where `conda list` reports 88 packages and this reports these 4.
    let input = r#"{
      "name": "base",
      "channels": ["conda-forge"],
      "dependencies": [
        "python=3.13",
        "conda[version='>=26.3.2']",
        "mamba[version='>=2.5.0']",
        "pip"
      ],
      "prefix": "/opt/conda"
    }"#;
    let names: Vec<String> = parse_conda_history(input)
        .expect("this fixture parses")
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(names, vec!["python", "conda", "mamba", "pip"]);
}

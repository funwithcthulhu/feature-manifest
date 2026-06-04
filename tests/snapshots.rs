mod common;

use common::{fixture_path, normalize, read_snapshot, run_command};
use serde_json::Value;

#[test]
fn markdown_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "markdown",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    assert_eq!(
        normalize(&output.stdout),
        read_snapshot("basic.markdown.md")
    );
}

#[test]
fn graph_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "graph",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    assert_eq!(normalize(&output.stdout), read_snapshot("basic.graph.mmd"));
}

#[test]
fn json_output_matches_snapshot() {
    let manifest_path = fixture_path("basic");
    let output = run_command(&[
        "json",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );
    assert_eq!(normalize(&output.stdout), read_snapshot("basic.json"));
}

#[test]
fn check_json_output_matches_snapshot() {
    let manifest_path = fixture_path("messy");
    let output = run_command(&[
        "check",
        "--format",
        "json",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    assert_eq!(normalize(&output.stdout), read_snapshot("messy.check.json"));
}

#[test]
fn check_github_output_matches_snapshot() {
    let manifest_path = fixture_path("messy");
    let output = run_command(&[
        "check",
        "--format",
        "github",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    assert_eq!(
        normalize(&output.stdout),
        read_snapshot("messy.check.github.txt")
    );
}

#[test]
fn check_sarif_output_matches_snapshot() {
    let manifest_path = fixture_path("messy");
    let output = run_command(&[
        "check",
        "--format",
        "sarif",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(!output.status.success());
    assert_eq!(
        normalize(&output.stdout),
        read_snapshot("messy.check.sarif.json")
    );
}

#[test]
fn sarif_reports_feature_and_metadata_source_spans() {
    let manifest_path = fixture_path("sarif-source-spans");
    let output = run_command(&[
        "check",
        "--format",
        "sarif",
        "--manifest-path",
        manifest_path
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);

    assert!(!output.status.success());

    let stdout = normalize(&output.stdout);
    let sarif = serde_json::from_str::<Value>(&stdout).expect("SARIF output should be JSON");
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("SARIF results should be an array");
    assert_eq!(results.len(), 2);

    assert_sarif_result(
        &results[0],
        "missing-metadata",
        "error",
        "Cargo.toml",
        10,
        "package `sarif-source-spans` feature `undocumented`: feature is defined in `[features]` but missing metadata; add an entry under `[package.metadata.feature-manifest]`.",
    );
    assert_sarif_result(
        &results[1],
        "unknown-metadata",
        "error",
        "Cargo.toml",
        17,
        "package `sarif-source-spans` feature `stale`: metadata exists for a feature that is not declared in `[features]`.",
    );
}

fn assert_sarif_result(
    result: &Value,
    rule_id: &str,
    level: &str,
    artifact_uri: &str,
    line: u64,
    message: &str,
) {
    let physical_location = &result["locations"][0]["physicalLocation"];

    assert_eq!(result["ruleId"], rule_id);
    assert_eq!(result["level"], level);
    assert_eq!(result["message"]["text"], message);
    assert_eq!(
        physical_location["artifactLocation"]["uri"],
        artifact_uri
    );
    assert_eq!(physical_location["region"]["startLine"], line);
}

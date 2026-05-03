mod common;

use std::fs;
use std::path::Path;

use common::{fixture_path, normalize, run_command, run_short_command};

#[test]
fn cli_reference_matches_generated_help_markdown() {
    let output = run_short_command(&["help-markdown"]);
    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );

    let docs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("cli.md");
    let docs = fs::read_to_string(docs_path).expect("failed to read generated CLI docs");

    assert_eq!(normalize(&output.stdout), normalize(docs.as_bytes()));
}

#[test]
fn published_json_schemas_are_valid_json_documents() {
    for schema_name in [
        "feature-manifest.v1.schema.json",
        "check-report.v1.schema.json",
    ] {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schemas")
            .join(schema_name);
        let schema = fs::read_to_string(schema_path).expect("failed to read JSON schema");
        let schema = serde_json::from_str::<serde_json::Value>(&schema)
            .expect("schema should parse as JSON");
        jsonschema::meta::validate(&schema).expect("schema should validate against its metaschema");
    }
}

#[test]
fn json_outputs_validate_against_published_schemas() {
    let manifest_output = run_command(&[
        "json",
        "-m",
        fixture_path("basic")
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        manifest_output.status.success(),
        "stderr:\n{}",
        normalize(&manifest_output.stderr)
    );
    validate_json_output(
        "feature-manifest.v1.schema.json",
        &normalize(&manifest_output.stdout),
    );

    let check_output = run_command(&[
        "check",
        "-f",
        "json",
        "-m",
        fixture_path("messy")
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(!check_output.status.success());
    validate_json_output(
        "check-report.v1.schema.json",
        &normalize(&check_output.stdout),
    );
}

fn validate_json_output(schema_name: &str, output: &str) {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join(schema_name);
    let schema = fs::read_to_string(schema_path).expect("failed to read JSON schema");
    let schema = serde_json::from_str::<serde_json::Value>(&schema).expect("schema should be JSON");
    let output = serde_json::from_str::<serde_json::Value>(output).expect("output should be JSON");

    jsonschema::validate(&schema, &output).expect("output should validate against schema");
}

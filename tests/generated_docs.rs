mod common;

use std::fs;
use std::path::Path;

use common::{fixture_path, normalize, run_command, run_short_command};
use feature_manifest::known_lint_codes;

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
fn lint_reference_matches_generated_markdown() {
    let output = run_short_command(&["lints", "--markdown"]);
    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );

    let docs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("lints.md");
    let docs = fs::read_to_string(docs_path).expect("failed to read generated lint docs");

    assert_eq!(normalize(&output.stdout), normalize(docs.as_bytes()));
}

#[test]
fn lint_codes_are_present_in_reference_surfaces() {
    let docs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("lints.md");
    let lint_docs = fs::read_to_string(docs_path).expect("failed to read lint docs");

    let json_schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("json-schema.md");
    let json_schema_docs =
        fs::read_to_string(json_schema_path).expect("failed to read JSON schema docs");

    let lints_output = run_short_command(&["list-lints"]);
    assert!(
        lints_output.status.success(),
        "stderr:\n{}",
        normalize(&lints_output.stderr)
    );
    let lints_stdout = normalize(&lints_output.stdout);

    for code in known_lint_codes() {
        assert!(
            lint_docs.contains(&format!("## `{code}`")),
            "docs/lints.md does not mention lint `{code}`"
        );
        assert!(
            json_schema_docs.contains("`code` | `string` | Stable lint code."),
            "docs/json-schema.md does not document the check-report issue code field"
        );
        assert!(
            lints_stdout.contains(code),
            "cargo fm list-lints output does not mention lint `{code}`"
        );
    }
}

#[test]
fn generated_markdown_keeps_readable_line_lengths() {
    for doc_name in ["cli.md", "lints.md"] {
        let docs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join(doc_name);
        let docs = fs::read_to_string(docs_path).expect("failed to read generated docs");

        for (index, line) in docs.lines().enumerate() {
            assert!(
                line.len() <= 100,
                "{doc_name}:{} has a raw line longer than 100 characters",
                index + 1
            );
        }
    }
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

#[test]
fn own_json_output_round_trips_through_committed_schema() {
    let output = run_command(&[
        "json",
        "-m",
        fixture_path("basic")
            .to_str()
            .expect("fixture path should be UTF-8"),
    ]);
    assert!(
        output.status.success(),
        "stderr:\n{}",
        normalize(&output.stderr)
    );

    validate_json_output(
        "feature-manifest.v1.schema.json",
        &normalize(&output.stdout),
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

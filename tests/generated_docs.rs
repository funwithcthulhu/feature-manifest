mod common;

use std::fs;
use std::path::Path;

use common::{normalize, run_short_command};

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
        serde_json::from_str::<serde_json::Value>(&schema).expect("schema should parse as JSON");
    }
}

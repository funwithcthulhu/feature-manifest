mod common;

use std::fs;
use std::path::Path;

use common::{fixture_path, normalize};
use feature_manifest::{FeatureRef, MetadataLayout, parse_manifest_str, validate};

#[test]
fn curated_compatibility_layouts_parse_and_validate() {
    let compat_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("compat");
    let mut checked = Vec::new();

    for entry in fs::read_dir(&compat_dir).expect("failed to read compatibility fixtures") {
        let entry = entry.expect("failed to read compatibility fixture entry");
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        let source = fs::read_to_string(&path).expect("failed to read compatibility fixture");
        let manifest = parse_manifest_str(&source, &path)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error:#}", path.display()));
        let report = validate(&manifest);
        assert!(
            !report.has_errors(),
            "{} produced validation errors:\n{}",
            path.display(),
            report
                .issues
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            !manifest.features.is_empty(),
            "{} should describe at least one feature",
            path.display()
        );
        checked.push(path);
    }

    checked.sort();
    assert_eq!(
        checked.len(),
        5,
        "expected five compatibility fixtures, found:\n{}",
        normalize(
            checked
                .iter()
                .map(|path| format!("{path:?}\n"))
                .collect::<String>()
                .as_bytes()
        )
    );
}

#[test]
fn legacy_feature_docs_table_remains_accepted() {
    let manifest = parse_manifest_str(
        r#"
[package]
name = "legacy-feature-docs"
version = "0.1.0"

[features]
default = []
serde = []

[package.metadata.feature-docs.features]
serde = "Enable serde support."
"#,
        "Cargo.toml",
    )
    .expect("legacy feature-docs metadata should parse");

    assert_eq!(manifest.metadata_table.as_deref(), Some("feature-docs"));
    assert_eq!(manifest.metadata_layout, MetadataLayout::Structured);
    assert!(manifest.features["serde"].has_metadata);
    assert!(!validate(&manifest).has_errors());
}

#[test]
fn realistic_feature_patterns_keep_current_parser_and_lint_behavior() {
    let path = fixture_path("realistic").join("Cargo.toml");
    let source = fs::read_to_string(&path).expect("failed to read realistic fixture");
    let manifest = parse_manifest_str(&source, &path).expect("realistic fixture should parse");

    assert_eq!(
        manifest.features["serde"].enables,
        vec![FeatureRef::Dependency {
            name: "serde".to_owned()
        }]
    );
    assert_eq!(
        manifest.features["runtime"].enables,
        vec![
            FeatureRef::Dependency {
                name: "tokio".to_owned()
            },
            FeatureRef::DependencyFeature {
                dependency: "tokio".to_owned(),
                feature: "rt".to_owned(),
                weak: false,
            },
        ]
    );

    let api_group = manifest
        .groups
        .iter()
        .find(|group| group.name == "api")
        .expect("api group should exist");
    assert_eq!(
        api_group.members,
        vec!["public-api".to_owned(), "internal-codegen".to_owned()]
    );
    assert!(manifest.features["public-api"].metadata.public);
    assert!(!manifest.features["internal-codegen"].metadata.public);
    assert!(manifest.metadata_only.contains_key("stale-feature"));

    let report = validate(&manifest);
    assert!(report.has_errors());
    assert!(report.issues.iter().any(|issue| {
        issue.code == "missing-metadata" && issue.feature.as_deref() == Some("undocumented")
    }));
    assert!(report.issues.iter().any(|issue| {
        issue.code == "missing-description" && issue.feature.as_deref() == Some("undocumented")
    }));
    assert!(report.issues.iter().any(|issue| {
        issue.code == "unknown-metadata" && issue.feature.as_deref() == Some("stale-feature")
    }));
}

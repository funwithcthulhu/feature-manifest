//! `feature-manifest` provides the library half of the `cargo-feature-manifest`
//! tool.
//!
//! It is designed for crate authors who want a structured way to describe Cargo
//! features today, validate those descriptions in CI, and render them into
//! documentation-friendly formats.
//!
//! Typical entry points:
//!
//! - [`load_workspace`] to discover workspace packages via `cargo metadata`.
//! - [`load_manifest`] to read a single `Cargo.toml`.
//! - [`validate`] to lint feature metadata.
//! - [`render_markdown`] to generate docs-friendly output.
//! - [`render_mermaid`] to visualize feature relationships.
//! - [`render_json`] to emit a versioned machine-readable schema.

mod discover;
mod json_output;
mod model;
mod parse;
mod render;
mod validate;

pub use discover::{PackageSelection, load_workspace, resolve_manifest_path};
pub use json_output::render_json;
pub use model::{
    Feature, FeatureGroup, FeatureManifest, FeatureMetadata, FeatureRef, WorkspaceManifest,
};
pub use parse::{load_manifest, parse_manifest_str, sync_manifest};
pub use render::{render_explain, render_markdown, render_mermaid};
pub use validate::{Issue, Severity, ValidationReport, validate};

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["serde", "tokio?/rt"]
serde = ["dep:serde"]
tokio = ["dep:tokio", "std"]
std = []
unstable = []

[package.metadata.feature-manifest]
serde = { description = "Enable serde support." }
tokio = { description = "Enable Tokio-backed APIs." }
std = { description = "Enable std support." }
unused = { description = "Not a real feature." }

[[package.metadata.feature-manifest.groups]]
name = "runtime"
members = ["tokio", "unstable"]
mutually_exclusive = true
"#;

    #[test]
    fn parses_typed_feature_references() {
        let manifest = parse_manifest_str(SAMPLE_MANIFEST, "Cargo.toml").unwrap();
        assert_eq!(manifest.package_name.as_deref(), Some("demo"));
        assert_eq!(manifest.features.len(), 4);
        assert_eq!(
            manifest.default_members,
            vec![
                FeatureRef::Feature {
                    name: "serde".to_owned()
                },
                FeatureRef::DependencyFeature {
                    dependency: "tokio".to_owned(),
                    feature: "rt".to_owned(),
                    weak: true
                }
            ]
        );
        assert_eq!(
            manifest.features["tokio"].enables,
            vec![
                FeatureRef::Dependency {
                    name: "tokio".to_owned()
                },
                FeatureRef::Feature {
                    name: "std".to_owned()
                }
            ]
        );
    }

    #[test]
    fn parses_structured_metadata_table() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
cli = []

[package.metadata.feature-manifest.features]
cli = "Enable the CLI layer."
"#,
            "Cargo.toml",
        )
        .unwrap();

        let cli = &manifest.features["cli"];
        assert!(cli.has_metadata);
        assert_eq!(
            cli.metadata.description.as_deref(),
            Some("Enable the CLI layer.")
        );
    }

    #[test]
    fn validation_reports_missing_and_stale_metadata() {
        let manifest = parse_manifest_str(SAMPLE_MANIFEST, "Cargo.toml").unwrap();
        let report = validate(&manifest);

        assert!(report.has_errors());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "missing-metadata"
                    && issue.feature.as_deref() == Some("unstable"))
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "unknown-metadata"
                    && issue.feature.as_deref() == Some("unused"))
        );
    }

    #[test]
    fn validation_reports_mutually_exclusive_default_conflicts() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["native-tls", "rustls"]
native-tls = []
rustls = []

[package.metadata.feature-manifest]
native-tls = { description = "Use native-tls." }
rustls = { description = "Use rustls." }

[[package.metadata.feature-manifest.groups]]
name = "tls"
members = ["native-tls", "rustls"]
mutually_exclusive = true
"#,
            "Cargo.toml",
        )
        .unwrap();

        let report = validate(&manifest);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "mutually-exclusive-default")
        );
    }

    #[test]
    fn markdown_hides_private_features_by_default_and_shows_default_summary() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["public-api"]
public-api = []
internal = []

[package.metadata.feature-manifest]
public-api = { description = "Stable public API surface." }
internal = { description = "Internal glue.", public = false }
"#,
            "Cargo.toml",
        )
        .unwrap();

        let workspace = WorkspaceManifest {
            root_manifest_path: "Cargo.toml".into(),
            packages: vec![manifest],
        };
        let markdown = render_markdown(&workspace, false);
        assert!(markdown.contains("Default feature set: `public-api`"));
        assert!(markdown.contains("public-api"));
        assert!(!markdown.contains("| `internal` |"));
        assert!(markdown.contains("internal/private feature(s) hidden"));
    }
}

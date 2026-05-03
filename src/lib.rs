//! `feature-manifest` provides the library half of the `cargo-feature-manifest`
//! tool.
//!
//! It is designed for crate authors who want a structured way to describe Cargo
//! features today, validate those descriptions in CI, and render them into
//! documentation-friendly formats.
//!
//! Typical entry points:
//!
//! - [`load_manifest`] to read a `Cargo.toml`.
//! - [`validate`] to lint feature metadata.
//! - [`render_markdown`] to generate a docs-friendly table.
//! - [`render_mermaid`] to visualize feature relationships.

mod manifest;
mod render;
mod validate;

pub use manifest::{
    FEATURE_DOCS_METADATA_TABLE, FEATURE_MANIFEST_METADATA_TABLE, Feature, FeatureGroup,
    FeatureManifest, FeatureMetadata, load_manifest, parse_manifest_str, resolve_manifest_path,
};
pub use render::{render_markdown, render_mermaid};
pub use validate::{Issue, Severity, ValidationReport, validate};

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"
[package]
name = "demo"
version = "0.1.0"

[features]
default = ["serde"]
serde = ["dep:serde"]
tokio = ["dep:tokio"]
unstable = []

[package.metadata.feature-manifest]
serde = { description = "Enable serde support." }
tokio = { description = "Enable Tokio-backed APIs." }
unused = { description = "Not a real feature." }

[[package.metadata.feature-manifest.groups]]
name = "runtime"
members = ["tokio", "unstable"]
mutually_exclusive = true
"#;

    #[test]
    fn parses_flat_metadata_table() {
        let manifest = parse_manifest_str(SAMPLE_MANIFEST, "Cargo.toml").unwrap();
        assert_eq!(manifest.package_name.as_deref(), Some("demo"));
        assert_eq!(manifest.features.len(), 3);
        assert!(manifest.features["serde"].has_metadata);
        assert_eq!(
            manifest.features["serde"].metadata.description.as_deref(),
            Some("Enable serde support.")
        );
        assert!(manifest.features["serde"].default_enabled);
        assert_eq!(manifest.metadata_only.len(), 1);
        assert!(manifest.metadata_only.contains_key("unused"));
        assert_eq!(manifest.groups.len(), 1);
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
    fn markdown_hides_private_features_by_default() {
        let manifest = parse_manifest_str(
            r#"
[package]
name = "demo"
version = "0.1.0"

[features]
public-api = []
internal = []

[package.metadata.feature-manifest]
public-api = { description = "Stable public API surface." }
internal = { description = "Internal glue.", public = false }
"#,
            "Cargo.toml",
        )
        .unwrap();

        let markdown = render_markdown(&manifest, false);
        assert!(markdown.contains("public-api"));
        assert!(!markdown.contains("| `internal` |"));
        assert!(markdown.contains("internal/private feature(s) hidden"));
    }
}
